//! One-shot migration of user grammar cards onto the v3 corpus (#503
//! content pass). Runs right after `grammar_v3.json` loads; safe to
//! re-run (idempotent: after the first pass no card references a missing
//! rule).
//!
//! Per card referencing a rule that the loaded corpus no longer has:
//! - merge twin → transfer the card onto the surviving rule (FSRS history
//!   keeps its schedule); if the user already holds a card for the
//!   successor, drop the migrating duplicate instead;
//! - deleted without successor (lexicon / lesson phrases / form
//!   umbrellas) → drop the card: nothing remains to review.
//!
//! Cards of other types and cards whose rules still exist are untouched.

use crate::dictionary::grammar::is_grammar_loaded;
use crate::dictionary::grammar_migration::migration_successor;
use crate::domain::{Card, GrammarRuleCard, OrigaError, User};
use crate::traits::UserRepository;
use tracing::info;
use ulid::Ulid;

/// Outcome of a migration pass, for logs and tests.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct MigrationReport {
    pub transferred: usize,
    pub dropped_duplicates: usize,
    pub dropped_deleted: usize,
}

impl MigrationReport {
    pub fn total(&self) -> usize {
        self.transferred + self.dropped_duplicates + self.dropped_deleted
    }
}

#[derive(Clone)]
pub struct MigrateGrammarCardsUseCase;

impl MigrateGrammarCardsUseCase {
    pub fn new() -> Self {
        Self
    }

    pub async fn execute<R: UserRepository>(
        &self,
        repository: &R,
    ) -> Result<MigrationReport, OrigaError> {
        if !is_grammar_loaded() {
            // No corpus → nothing to migrate against; the loader calls us
            // right after a successful load, so this is a programming
            // error guard rather than a user-facing state.
            return Err(OrigaError::GrammarNotLoaded);
        }

        let mut user = repository
            .get_current_user()
            .await?
            .ok_or(OrigaError::CurrentUserNotExist)?;

        let report = migrate_user_cards(&mut user)?;
        if report.total() > 0 {
            repository.save(&user).await?;
            info!(
                transferred = report.transferred,
                dropped_duplicates = report.dropped_duplicates,
                dropped_deleted = report.dropped_deleted,
                "Grammar v3 card migration"
            );
        }
        Ok(report)
    }
}

impl Default for MigrateGrammarCardsUseCase {
    fn default() -> Self {
        Self::new()
    }
}

/// Pure core: rewrite/drop the user's stale grammar cards in place.
/// Public for unit tests; the corpus lookup uses the loaded store.
pub fn migrate_user_cards(user: &mut User) -> Result<MigrationReport, OrigaError> {
    let mut report = MigrationReport::default();

    let live_grammar_ids: std::collections::HashSet<Ulid> =
        crate::dictionary::grammar::iter_grammar_rules()
            .map(|rule| *rule.rule_id())
            .collect();

    // Snapshot first: we mutate the map while iterating decisions.
    let grammar_cards: Vec<(Ulid, Ulid)> = user
        .knowledge_set()
        .study_cards()
        .iter()
        .filter_map(|(card_id, study)| match study.card() {
            Card::Grammar(grc) => Some((*card_id, *grc.rule_id())),
            _ => None,
        })
        .collect();

    // Successors that already have their own card: a transfer onto them
    // would duplicate a rule inside one knowledge set.
    let mut existing_rule_ids: std::collections::HashSet<Ulid> = user
        .knowledge_set()
        .study_cards()
        .values()
        .filter_map(|study| match study.card() {
            Card::Grammar(grc) => Some(*grc.rule_id()),
            _ => None,
        })
        .collect();

    for (card_id, rule_id) in grammar_cards {
        if live_grammar_ids.contains(&rule_id) {
            continue;
        }
        match migration_successor(&rule_id) {
            Some(successor) if existing_rule_ids.contains(&successor) => {
                user.delete_card(card_id)?;
                report.dropped_duplicates += 1;
            },
            Some(successor) => {
                let replacement = Card::Grammar(GrammarRuleCard::new(successor)?);
                user.update_card_content(card_id, replacement)?;
                existing_rule_ids.insert(successor);
                report.transferred += 1;
            },
            None => {
                user.delete_card(card_id)?;
                report.dropped_deleted += 1;
            },
        }
    }
    Ok(report)
}
