//! Startup hook for the one-time deck cleanup (uwuwu#1463).

use leptos::task::spawn_local;

use crate::repository::HybridUserRepository;

pub fn spawn(repository: HybridUserRepository) {
    spawn_local(async move {
        if let Err(e) = super::dictionary::ensure_tokenizer_loaded().await {
            tracing::warn!("Word identity migration skipped: tokenizer unavailable: {e}");
            return;
        }
        match origa::use_cases::MigrateWordIdentityForExistingUsersUseCase::new(&repository)
            .execute()
            .await
        {
            Ok(report) if report.cards_rebased > 0 || report.cards_removed > 0 => tracing::info!(
                rebased = report.cards_rebased,
                removed = report.cards_removed,
                "Historical word cards migrated to dictionary lemmas"
            ),
            Ok(_) => {},
            Err(e) => tracing::warn!("Word identity migration failed: {e}"),
        }
    });
}
