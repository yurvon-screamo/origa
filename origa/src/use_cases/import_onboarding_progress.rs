//! Progress contract of the onboarding sets import. The UI drives its
//! import indicator from these events; the use case emits them at
//! phase boundaries and in bounded chunks, awaiting the future each event
//! returns — that await is where the host (WASM) yields to the browser and
//! a pending frame actually renders.

/// Pipeline phase of an [`ImportProgressEvent`]. Order matches execution
/// order; the save phase always closes the import.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImportPhase {
    /// Downloading well-known set bodies from the CDN.
    FetchSets,
    /// Tokenizing set word lists and creating vocabulary cards.
    Vocabulary,
    /// Creating grammar rules for the imported levels.
    Grammar,
    /// Creating kanji cards (with companion vocabulary) up to the target level.
    Kanji,
    /// Persisting the imported user (local store + remote sync).
    Saving,
}

/// One progress tick: how far the current [`ImportPhase`] has moved.
/// `total == 1` phases (Saving) render as an indeterminate "in progress".
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ImportProgressEvent {
    pub phase: ImportPhase,
    pub done: usize,
    pub total: usize,
}
