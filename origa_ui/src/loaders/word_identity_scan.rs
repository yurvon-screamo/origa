//! Startup duplicate scan (uwuwu#1463). Read-only: never saves the user record.

use leptos::task::spawn_local;

use crate::repository::HybridUserRepository;

pub fn spawn(repository: HybridUserRepository) {
    spawn_local(async move {
        match origa::use_cases::ScanWordIdentityUseCase::new(&repository)
            .execute()
            .await
        {
            Ok(report) if report.duplicate_cards() > 0 => tracing::info!(
                groups = report.group_count(),
                duplicates = report.duplicate_cards(),
                "Duplicate vocabulary groups in deck"
            ),
            Ok(_) => {},
            Err(e) => tracing::warn!("Duplicate card scan failed: {e}"),
        }
    });
}
