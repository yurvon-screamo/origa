//! Anonymous installation identifier for Sentry release-health sessions.
//!
//! A UUID v4 generated on first launch and persisted to
//! `<app_data_dir>/install.json`. It is set as the Sentry scope user id
//! before the release-health session starts, so Sentry "Users" counts
//! unique installations (see the ADR-036 addendum). The identifier is
//! random, contains no personal data, and is never linked to an account.
//!
//! The file is a statistics aid, not load-bearing state: corrupted or
//! unwritable storage degrades to `None` (session without a distinct_id)
//! instead of failing the launch.

use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

const INSTALL_ID_FILE: &str = "install.json";

#[derive(Serialize, Deserialize)]
struct InstallIdFile {
    install_id: String,
}

/// Reads the persisted installation UUID, creating it on first launch.
///
/// Returns `None` when the identifier cannot be persisted (unwritable
/// data dir): callers then skip setting the Sentry user, so an ephemeral
/// per-launch UUID does not inflate the Users count.
pub fn read_or_create(data_dir: &Path) -> Option<Uuid> {
    let path = data_dir.join(INSTALL_ID_FILE);
    if let Some(id) = read_existing(&path) {
        return Some(id);
    }

    let install_id = Uuid::new_v4();
    let json = match serde_json::to_string(&InstallIdFile {
        install_id: install_id.to_string(),
    }) {
        Ok(json) => json,
        Err(e) => {
            tracing::warn!("[install-id] failed to serialize install id: {e}");
            return None;
        },
    };
    if let Err(e) = fs::create_dir_all(data_dir) {
        tracing::warn!(
            "[install-id] failed to create data dir {}: {e}",
            data_dir.display()
        );
        return None;
    }
    match fs::write(&path, json) {
        Ok(()) => Some(install_id),
        Err(e) => {
            tracing::warn!("[install-id] failed to persist {INSTALL_ID_FILE}: {e}");
            None
        },
    }
}

/// Parses the identifier from an existing file. `None` when the file is
/// missing, corrupted, or holds a non-UUID value — the caller regenerates.
fn read_existing(path: &Path) -> Option<Uuid> {
    let raw = fs::read_to_string(path).ok()?;
    let parsed: InstallIdFile = serde_json::from_str(&raw).ok()?;
    Uuid::parse_str(&parsed.install_id).ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn creates_install_id_when_file_is_missing() {
        let dir = tempfile::tempdir().unwrap();

        let id = read_or_create(dir.path());

        let id = id.expect("first launch must create an install id");
        let file_contents = fs::read_to_string(dir.path().join(INSTALL_ID_FILE)).unwrap();
        assert!(file_contents.contains(&id.to_string()));
    }

    #[test]
    fn returns_the_same_install_id_on_subsequent_reads() {
        let dir = tempfile::tempdir().unwrap();
        let first = read_or_create(dir.path()).unwrap();

        let second = read_or_create(dir.path()).unwrap();

        assert_eq!(first, second);
    }

    #[test]
    fn regenerates_install_id_when_file_is_corrupted() {
        let dir = tempfile::tempdir().unwrap();
        let original = read_or_create(dir.path()).unwrap();
        fs::write(dir.path().join(INSTALL_ID_FILE), "not a json").unwrap();

        let regenerated = read_or_create(dir.path()).unwrap();

        assert_ne!(original, regenerated);
    }

    #[test]
    fn creates_missing_data_directory() {
        let dir = tempfile::tempdir().unwrap();
        let data_dir = dir.path().join("nested/app-data");

        let id = read_or_create(&data_dir);

        assert!(id.is_some());
        assert!(data_dir.is_dir());
    }
}
