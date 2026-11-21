use std::collections::HashSet;
use std::future::Future;

use futures::stream::StreamExt;
use tracing::{debug, info, warn};
use web_time::Instant;

use crate::domain::resolve_set_path;
use crate::domain::{JapaneseLevel, OrigaError, User, WellKnownSet};
use crate::traits::{CdnProvider, UserRepository};
use crate::use_cases::import_onboarding_progress::{ImportPhase, ImportProgressEvent};
use crate::use_cases::import_onboarding_steps::{
    import_grammar_for_levels, import_kanji_up_to_level, import_vocabulary_from_sets,
};
use crate::use_cases::seed_counters_into_user;

/// Concurrent CDN downloads per fetch batch. Bounded to keep a mobile
/// WebView's Cache API + connection pool healthy; with 184 sets the batch
/// round trips still shrink the fetch phase ~8×.
const FETCH_CONCURRENCY: usize = 8;
/// Set downloads per progress tick in the fetch phase.
const FETCH_PROGRESS_CHUNK: usize = 8;

#[derive(Debug)]
pub struct ImportOnboardingResult {
    pub imported_set_ids: Vec<String>,
    pub created_vocabulary: usize,
    pub created_kanji: usize,
    pub created_grammar: usize,
    pub created_counters: usize,
    pub skipped_duplicates: usize,
    pub skipped_no_translation: usize,
}

#[derive(Clone)]
pub struct ImportOnboardingSetsUseCase<'a, R: UserRepository, C: CdnProvider> {
    repository: &'a R,
    cdn: &'a C,
}

impl<'a, R: UserRepository, C: CdnProvider> ImportOnboardingSetsUseCase<'a, R, C> {
    pub fn new(repository: &'a R, cdn: &'a C) -> Self {
        Self { repository, cdn }
    }

    pub async fn execute(
        &self,
        user: User,
        set_ids: Vec<String>,
        target_level: JapaneseLevel,
    ) -> Result<ImportOnboardingResult, OrigaError> {
        self.execute_with_progress(user, set_ids, target_level, |_| std::future::ready(()))
            .await
    }

    /// Same pipeline as [`Self::execute`], reporting progress through
    /// `on_progress`. Every event's future is awaited before the next chunk
    /// starts — a UI host yields to the browser there so the indicator
    /// renders; tests pass a ready future.
    pub async fn execute_with_progress<F, Fut>(
        &self,
        mut user: User,
        set_ids: Vec<String>,
        target_level: JapaneseLevel,
        on_progress: F,
    ) -> Result<ImportOnboardingResult, OrigaError>
    where
        F: Fn(ImportProgressEvent) -> Fut,
        Fut: Future<Output = ()>,
    {
        debug!(user_id = %user.id(), set_count = set_ids.len(), ?target_level, "Starting onboarding sets import");

        let native_language = *user.native_language();
        let import_started = Instant::now();

        let phase_started = Instant::now();
        let sets = self.load_sets_via_cdn(&set_ids, &on_progress).await?;
        let fetch_ms = phase_started.elapsed().as_millis() as u64;

        let mut result = ImportOnboardingResult {
            imported_set_ids: Vec::new(),
            created_vocabulary: 0,
            created_kanji: 0,
            created_grammar: 0,
            created_counters: 0,
            skipped_duplicates: 0,
            skipped_no_translation: 0,
        };

        let mut created_kanji_chars: HashSet<String> = HashSet::new();
        let mut grammar_levels: HashSet<JapaneseLevel> = HashSet::new();

        // Thousands of cards are created below; the bulk bracket keeps each
        // create O(1) (index-backed uniqueness + deferred stats recalc) —
        // per-card full scans made N2-scale imports quadratic.
        user.begin_bulk_import();

        let phase_started = Instant::now();
        import_vocabulary_from_sets(
            &mut user,
            sets,
            &native_language,
            &mut result,
            &mut grammar_levels,
            &on_progress,
        )
        .await;
        let vocabulary_ms = phase_started.elapsed().as_millis() as u64;

        // Pull in every level at or below the target so a user picking N3 also
        // gets N5/N4/N2/... grammar rules and kanji they should already know,
        // regardless of which sets they imported.
        for level in JapaneseLevel::ALL {
            if level <= target_level {
                grammar_levels.insert(level);
            }
        }

        let phase_started = Instant::now();
        import_grammar_for_levels(&mut user, &grammar_levels, &mut result, &on_progress).await;
        let grammar_ms = phase_started.elapsed().as_millis() as u64;

        let phase_started = Instant::now();
        import_kanji_up_to_level(
            &mut user,
            target_level,
            &mut result,
            &mut created_kanji_chars,
            &on_progress,
        )
        .await;
        let kanji_ms = phase_started.elapsed().as_millis() as u64;

        // Счётные суффиксы ≤ целевого уровня (issue #415): сидируются при
        // РЕАЛЬНОМ импорте сетов (план: «юзер завершил импорт сетов»), чтобы
        // пустой импорт не менял контракт скоринга «ноль карт». Пропущенные
        // при пустом импорте счётчики добирает миграция первого старта.
        if !set_ids.is_empty() {
            match seed_counters_into_user(&mut user, target_level) {
                Ok(n) => result.created_counters = n,
                Err(e) => warn!(error = ?e, "Counter seeding failed during onboarding import"),
            }
        }

        user.end_bulk_import();

        user.mark_sets_as_imported(set_ids);

        on_progress(ImportProgressEvent {
            phase: ImportPhase::Saving,
            done: 0,
            total: 1,
        })
        .await;
        let phase_started = Instant::now();
        self.repository.save_sync(&user).await?;
        let save_ms = phase_started.elapsed().as_millis() as u64;

        info!(
            user_id = %user.id(),
            vocabulary = result.created_vocabulary,
            kanji = result.created_kanji,
            grammar = result.created_grammar,
            duplicates = result.skipped_duplicates,
            fetch_ms = fetch_ms,
            vocabulary_ms = vocabulary_ms,
            grammar_ms = grammar_ms,
            kanji_ms = kanji_ms,
            save_ms = save_ms,
            total_ms = import_started.elapsed().as_millis() as u64,
            "Onboarding sets import completed"
        );

        Ok(result)
    }

    /// Downloads every set body from the CDN. Requests run in bounded
    /// concurrent batches ([`FETCH_CONCURRENCY`]); results and the reported
    /// error are restored to input order, so observable behavior matches the
    /// previous sequential loop (the first failing set in input order wins).
    async fn load_sets_via_cdn<F, Fut>(
        &self,
        set_ids: &[String],
        on_progress: &F,
    ) -> Result<Vec<(String, WellKnownSet)>, OrigaError>
    where
        F: Fn(ImportProgressEvent) -> Fut,
        Fut: Future<Output = ()>,
    {
        #[derive(serde::Deserialize)]
        struct SetData {
            level: JapaneseLevel,
            words: Vec<String>,
        }

        let total = set_ids.len();
        let mut bodies: Vec<Option<(String, WellKnownSet)>> = vec![None; total];
        let mut errors: Vec<Option<OrigaError>> = vec![None; total];

        let requests = set_ids.iter().enumerate().map(|(index, id)| {
            let id = id.clone();
            async move {
                let path = resolve_set_path(&id);
                let outcome = async {
                    let json = self.cdn.fetch_text(&path).await?;
                    let data: SetData = serde_json::from_str(&json).map_err(|e| {
                        OrigaError::WellKnownSetParseError {
                            reason: format!("Error parsing {}: {}", id, e),
                        }
                    })?;
                    Ok((id, WellKnownSet::new(data.level, data.words)))
                }
                .await;
                (index, outcome)
            }
        });

        let mut fetched = futures::stream::iter(requests).buffer_unordered(FETCH_CONCURRENCY);
        let mut done = 0usize;
        while let Some((index, outcome)) = fetched.next().await {
            match outcome {
                Ok((id, set)) => {
                    bodies[index] = Some((id, set));
                    done += 1;
                    if done == total || done % FETCH_PROGRESS_CHUNK == 0 {
                        on_progress(ImportProgressEvent {
                            phase: ImportPhase::FetchSets,
                            done,
                            total,
                        })
                        .await;
                    }
                },
                Err(e) => errors[index] = Some(e),
            }
        }

        if let Some(error) = errors.iter().flatten().next() {
            return Err(error.clone());
        }

        bodies
            .into_iter()
            .map(|slot| {
                slot.ok_or(OrigaError::RepositoryError {
                    reason: "set body missing without a fetch error".to_string(),
                })
            })
            .collect()
    }
}
