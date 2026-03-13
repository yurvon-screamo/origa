use leptos::prelude::*;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::OnceLock;

static SYNC_ACTIVE: OnceLock<AtomicBool> = OnceLock::new();
static LAST_SYNC_TIMESTAMP: OnceLock<AtomicU64> = OnceLock::new();
static SYNC_VERSION: OnceLock<AtomicU64> = OnceLock::new();

fn is_sync_active() -> bool {
    SYNC_ACTIVE
        .get_or_init(|| AtomicBool::new(false))
        .load(Ordering::Relaxed)
}

fn set_sync_active(value: bool) {
    SYNC_ACTIVE
        .get_or_init(|| AtomicBool::new(false))
        .store(value, Ordering::Relaxed);
}

pub fn get_last_sync_timestamp() -> u64 {
    LAST_SYNC_TIMESTAMP
        .get_or_init(|| AtomicU64::new(0))
        .load(Ordering::Relaxed)
}

fn set_last_sync_timestamp(value: u64) {
    LAST_SYNC_TIMESTAMP
        .get_or_init(|| AtomicU64::new(0))
        .store(value, Ordering::Relaxed);
}

pub fn get_sync_version() -> u64 {
    SYNC_VERSION
        .get_or_init(|| AtomicU64::new(0))
        .load(Ordering::Relaxed)
}

pub fn increment_sync_version() {
    SYNC_VERSION
        .get_or_init(|| AtomicU64::new(0))
        .fetch_add(1, Ordering::Relaxed);
}

pub fn reset_sync_context() {
    set_sync_active(false);
    set_last_sync_timestamp(0);
}

#[derive(Clone, Copy)]
pub struct SyncContext {
    pub sync_trigger: RwSignal<u64>,
    pub is_syncing: RwSignal<bool>,
    pub sync_error: RwSignal<Option<String>>,
}

impl SyncContext {
    pub fn new() -> Self {
        Self {
            sync_trigger: RwSignal::new(0),
            is_syncing: RwSignal::new(false),
            sync_error: RwSignal::new(None),
        }
    }

    pub fn trigger_sync(&self) {
        self.sync_trigger.set(self.sync_trigger.get_untracked() + 1);
    }

    pub fn start_sync(&self) {
        set_sync_active(true);
        self.is_syncing.set(true);
        self.sync_error.set(None);
    }

    pub fn complete_sync(&self) {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        set_last_sync_timestamp(now);
        increment_sync_version();
        self.is_syncing.set(false);
        self.trigger_sync();
    }

    pub fn fail_sync(&self, error: String) {
        self.is_syncing.set(false);
        self.sync_error.set(Some(error));
    }

    pub fn stop_background_sync(&self) {
        set_sync_active(false);
    }

    pub fn is_background_active() -> bool {
        is_sync_active()
    }
}

impl Default for SyncContext {
    fn default() -> Self {
        Self::new()
    }
}
