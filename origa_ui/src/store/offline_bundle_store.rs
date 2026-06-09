use leptos::prelude::*;

use crate::loaders::precache_loader::PreCacheProgress;

#[derive(Clone, Copy, PartialEq)]
pub enum CardCacheState {
    Idle,
    Running,
    Complete,
}

/// Manages card cache state. Bundle download state is managed locally
/// in OfflineBundleCard since it's only needed in the profile page.
#[derive(Clone)]
pub struct OfflineBundleStore {
    pub card_cache_state: RwSignal<CardCacheState>,
    pub card_cache_progress: RwSignal<PreCacheProgress>,
}

impl OfflineBundleStore {
    pub fn new() -> Self {
        Self {
            card_cache_state: RwSignal::new(CardCacheState::Idle),
            card_cache_progress: RwSignal::new(PreCacheProgress::default()),
        }
    }
}

impl Default for OfflineBundleStore {
    fn default() -> Self {
        Self::new()
    }
}
