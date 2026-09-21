//! Client-side sync bookkeeping for the hybrid user repository.
//!
//! A materialized `User` with a large knowledge set is expensive in WASM
//! (megabytes of JSON to inflate, parse and re-serialize on a single
//! thread), so the repository avoids full sync cycles when nothing changed.
//! This module holds the pure decision state for that short-circuit:
//! a fingerprint of the last synchronized remote row plus a dirty flag with
//! an epoch counter that makes concurrent mutations visible.
//!
//! See ADR-045 for the full sync design and threat model.

use serde::{Deserialize, Serialize};

/// Sync bookkeeping persisted beside the local user record.
///
/// Invariants:
/// - `dirty == true` forces the next sync down the full merge path.
/// - `dirty_epoch` increments on every [`SyncMeta::mark_dirty`] call; the
///   sync orchestration captures it **after its own** `mark_dirty` and
///   passes the captured value to [`SyncMeta::record_sync`], which refuses
///   to clear the flag when any further mutation happened in the sync
///   window (lost-update protection).
/// - `last_synced_fingerprint` is computed from the server's read-back row
///   shape: after a push it is reconstructed from the pushed payload
///   (skeleton ∪ wire ∪ id — ADR-060; the server stores the pushed JSON
///   verbatim and `updated_at` is the client's own wire-bump), and on any
///   full check it comes from the fetched row bytes. The delta-probe
///   valve bounds any reconstruction drift.
/// - `last_synced_record_id` + `last_seen_updated_at` identify the exact
///   server row the fingerprint belongs to, enabling the delta probe (a
///   skip-path without the multi-megabyte download). The new fields carry
///   `#[serde(default)]`: records persisted before this feature upgrade
///   transparently, and older readers ignore unknown fields (downgrade
///   safe — do not add `deny_unknown_fields`).
/// - `probe_skips_since_full` counts consecutive delta-probe skips; once
///   the count reaches [`PROBE_SKIP_FULL_CHECK_INTERVAL`], the next sync
///   trigger ignores the probe's verdict and runs one full check — the
///   bounded safety valve for every false-negative class the probe can
///   have (cross-device clock skew, timestamp collisions, format drift,
///   server-side row deletion, post-push reconstruction drift — see
///   ADR-060).
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SyncMeta {
    pub last_synced_fingerprint: Option<String>,
    pub dirty: bool,
    pub dirty_epoch: u64,
    #[serde(default)]
    pub last_synced_record_id: Option<i64>,
    #[serde(default)]
    pub last_seen_updated_at: Option<String>,
    #[serde(default)]
    pub probe_skips_since_full: u32,
}

/// Consecutive delta-probe skips after which one full check runs anyway.
/// Bounds the staleness of every probe false-negative class to this many
/// sync triggers, without injecting a clock into the orchestration.
pub const PROBE_SKIP_FULL_CHECK_INTERVAL: u32 = 20;

impl SyncMeta {
    /// Initial state for a fresh install or a pre-sync-feature upgrade:
    /// no fingerprint and dirty, so the first sync is always a full one
    /// (fail-closed).
    pub fn unsynced() -> Self {
        Self {
            last_synced_fingerprint: None,
            dirty: true,
            dirty_epoch: 0,
            ..Default::default()
        }
    }

    /// Whether the sync orchestration may skip the full merge path for a
    /// remote row with `remote_fingerprint`.
    pub fn should_skip(&self, remote_fingerprint: &str) -> bool {
        !self.dirty && self.last_synced_fingerprint.as_deref() == Some(remote_fingerprint)
    }

    /// The delta-probe target: the exact server row to re-check, when the
    /// state is clean and both probe fields are known. `None` disables the
    /// probe — fresh installs and pre-upgrade records take the full path
    /// until the first full sync records the fields (fail-closed on the
    /// new mechanics).
    pub fn probe_target(&self) -> Option<(i64, &str)> {
        if self.dirty {
            return None;
        }
        match (
            self.last_synced_record_id,
            self.last_seen_updated_at.as_deref(),
        ) {
            (Some(record_id), Some(updated_at)) => Some((record_id, updated_at)),
            _ => None,
        }
    }

    /// Records a local mutation: the next sync must take the full path.
    /// Increments `dirty_epoch` so an in-flight sync notices the mutation.
    pub fn mark_dirty(&mut self) {
        self.dirty = true;
        self.dirty_epoch += 1;
    }

    /// Records a successful full sync against the server row with
    /// `remote_fingerprint`.
    ///
    /// `observed_epoch` must be the value of `dirty_epoch` captured by the
    /// sync orchestration **after its own** `mark_dirty`: when the epoch
    /// still matches, no other mutation happened in the sync window and the
    /// dirty flag may clear; otherwise a concurrent `mark_dirty` (e.g. a
    /// card rated while the sync was pushing) keeps the flag set so the
    /// next sync re-merges and pushes the newer local state.
    pub fn record_sync(&mut self, remote_fingerprint: String, observed_epoch: u64) {
        self.last_synced_fingerprint = Some(remote_fingerprint);
        if self.dirty_epoch == observed_epoch {
            self.dirty = false;
        }
    }

    /// Records the delta-probe bookkeeping: the exact row the fingerprint
    /// belongs to and its `updated_at` stamp. Called on every path that
    /// has seen a full row (the fingerprint-skip branch included — that is
    /// what makes a false probe alarm self-healing) and after a push (the
    /// stamp is the pushed wire-bump, passed through verbatim so the
    /// probe's lexicographic `$gt` compares the exact stored bytes —
    /// ADR-060). Resets the skip counter.
    pub fn record_probe_row(&mut self, record_id: i64, updated_at: String) {
        self.last_synced_record_id = Some(record_id);
        self.last_seen_updated_at = Some(updated_at);
        self.probe_skips_since_full = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn synced_meta(fingerprint: &str) -> SyncMeta {
        SyncMeta {
            last_synced_fingerprint: Some(fingerprint.to_string()),
            dirty: false,
            dirty_epoch: 3,
            ..Default::default()
        }
    }

    #[test]
    fn unsynced_meta_never_skips() {
        // Arrange / Act / Assert: a fresh install has no fingerprint and is
        // dirty, so the first sync is always full regardless of the remote.
        let meta = SyncMeta::unsynced();
        assert!(!meta.should_skip("anything"));
        assert!(meta.dirty);
    }

    #[test]
    fn default_meta_never_skips() {
        // A missing persisted record deserializes to default (None fp,
        // not dirty) — the orchestrator must treat it as unsynced, but the
        // type itself guarantees skip requires a fingerprint.
        assert!(!SyncMeta::default().should_skip("x"));
    }

    #[test]
    fn clean_matching_fingerprint_skips() {
        let meta = synced_meta("fp1");
        assert!(meta.should_skip("fp1"));
    }

    #[test]
    fn changed_remote_fingerprint_does_not_skip() {
        let meta = synced_meta("fp1");
        assert!(!meta.should_skip("fp2"));
    }

    #[test]
    fn dirty_meta_does_not_skip_even_with_matching_fingerprint() {
        let mut meta = synced_meta("fp1");
        meta.mark_dirty();
        assert!(!meta.should_skip("fp1"));
    }

    #[test]
    fn record_sync_clears_dirty_when_epoch_unchanged() {
        let mut meta = SyncMeta::unsynced();
        meta.mark_dirty();
        let observed = meta.dirty_epoch;

        meta.record_sync("fp1".to_string(), observed);

        assert!(!meta.dirty);
        assert_eq!(meta.last_synced_fingerprint.as_deref(), Some("fp1"));
        assert!(meta.should_skip("fp1"));
    }

    #[test]
    fn record_sync_keeps_dirty_when_mutated_during_sync_window() {
        // The lost-update race: a card rated while the sync's push is in
        // flight must survive — the epoch moved past the observed value.
        let mut meta = SyncMeta::unsynced();
        meta.mark_dirty();
        let observed = meta.dirty_epoch;
        meta.mark_dirty(); // concurrent mutation during the sync window

        meta.record_sync("fp1".to_string(), observed);

        assert!(meta.dirty, "concurrent mutation must keep the flag set");
        assert_eq!(meta.last_synced_fingerprint.as_deref(), Some("fp1"));
        assert!(!meta.should_skip("fp1"));
    }

    #[test]
    fn crash_before_record_sync_keeps_meta_dirty() {
        // Simulates dying after the PATCH but before the meta write: the
        // in-memory copy is dirty and the next sync is a full one (an
        // acceptable extra full sync, documented in ADR-045).
        let mut meta = SyncMeta::unsynced();
        meta.mark_dirty();
        // no record_sync call — e.g. the process died
        assert!(meta.dirty);
        assert!(!meta.should_skip("fp1"));
    }

    #[test]
    fn mark_dirty_increments_epoch_every_time() {
        let mut meta = synced_meta("fp1");
        let before = meta.dirty_epoch;
        meta.mark_dirty();
        meta.mark_dirty();
        assert_eq!(meta.dirty_epoch, before + 2);
    }

    #[test]
    fn pre_probe_meta_json_upgrades_transparently() {
        // A record persisted before the delta probe existed decodes with
        // the probe fields defaulted: the probe stays off until the first
        // full sync records them (fail-closed on the new mechanics).
        let legacy = r#"{
            "last_synced_fingerprint": "fp1",
            "dirty": false,
            "dirty_epoch": 3
        }"#;
        let meta: SyncMeta = serde_json::from_str(legacy).expect("legacy meta decodes");
        assert_eq!(meta.last_synced_fingerprint.as_deref(), Some("fp1"));
        assert!(!meta.dirty);
        assert_eq!(meta.probe_target(), None, "no probe fields → no probe");
        assert_eq!(meta.probe_skips_since_full, 0);
    }

    #[test]
    fn probe_target_requires_clean_state_and_both_fields() {
        let mut meta = synced_meta("fp1");
        assert_eq!(meta.probe_target(), None, "no probe fields yet");

        meta.record_probe_row(7, "2026-09-20T10:00:00+00:00".to_string());
        assert_eq!(
            meta.probe_target(),
            Some((7, "2026-09-20T10:00:00+00:00")),
            "clean meta with both fields probes"
        );

        meta.mark_dirty();
        assert_eq!(meta.probe_target(), None, "dirty meta never probes");
    }

    #[test]
    fn record_probe_row_resets_the_skip_counter() {
        let mut meta = synced_meta("fp1");
        meta.probe_skips_since_full = 19;
        meta.record_probe_row(7, "2026-09-20T10:00:00+00:00".to_string());
        assert_eq!(meta.probe_skips_since_full, 0);
    }

    #[test]
    fn new_fields_are_downgrade_safe() {
        // Older readers must be able to ignore the probe fields — the
        // serialized form keeps working if this ever needs asserting in a
        // cross-version test. Here: roundtrip preserves everything.
        let mut meta = synced_meta("fp1");
        meta.record_probe_row(42, "2026-09-20T10:00:00+00:00".to_string());
        let json = serde_json::to_string(&meta).expect("serialize");
        let back: SyncMeta = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(meta, back);
    }
}
