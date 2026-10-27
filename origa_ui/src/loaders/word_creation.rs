//! The single UI entry point for the single-word `from_text` creation
//! flow (the words-page `AddWordModal` and the translator quick-add).
//! The batch paths (`add_words_preview_modal_state`, set import) keep
//! their own gate+use-case composition — a different use-case contract.
//!
//! The tokenizer gate is a UI concern (a CDN dictionary load) and cannot
//! live in the domain use case — but it must fire before EVERY creation
//! path: `VocabularyCard::from_text` silently swallows `TokenizerError`,
//! so an un-gated call returns an empty result and the word is lost with
//! zero signals (the #865 plan-review High). UI policies (error states
//! vs the zero-state button) stay with the call sites.

use origa::domain::OrigaError;
use origa::use_cases::{CreateVocabularyCardResult, CreateVocabularyCardUseCase};

use crate::loaders::dictionary::ensure_tokenizer_loaded;
use crate::repository::HybridUserRepository;

/// Create a vocabulary card for `word` (the dictionary form is the
/// caller's choice): gate on tokenizer readiness, then run the domain
/// use case. Duplicate/no-translation skips come back inside the result
/// — the use case logs them. No behavioral test: the gate pulls the CDN
/// tokenizer dictionary into the suite (a heavy network dependency); the
/// glue is covered piecewise (gate — warming/modal flows, use case — 9
/// native journey tests).
pub async fn add_word_to_deck(
    repository: &HybridUserRepository,
    word: &str,
) -> Result<CreateVocabularyCardResult, OrigaError> {
    ensure_tokenizer_loaded().await?;
    CreateVocabularyCardUseCase::new(repository)
        .execute(word.to_string())
        .await
}
