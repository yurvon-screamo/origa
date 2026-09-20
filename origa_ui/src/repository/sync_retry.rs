//! One-shot retry for user-record sync operations.
//!
//! The Railway edge intermittently answers `400/502 "upstream error"`
//! without the request ever reaching the container — a platform routing
//! hiccup documented across Railway's community threads, typically around
//! redeploys (this service redeploys on every master push). A retry a
//! second and a half later lands after the hiccup window.
//!
//! Retrying is safe: both sync operations (`merge_current_user`, the
//! post-merge fetch) are idempotent by design — the ADR-045 full path
//! re-runs cleanly, and the fetch is a read.

use std::future::Future;

use origa::domain::OrigaError;

/// Delay before the single retry.
pub(crate) const SYNC_RETRY_DELAY_MS: u64 = 1500;

/// Whether a sync failure deserves the single retry. Follows the ADR-053
/// retry doctrine (`should_retry_after_error` in routes.rs): a proven
/// idle-timeout stall means the neighbour already timed out — retrying is
/// pure latency, never re-run it. An expired session needs a re-login, not
/// a second roundtrip. Everything else (the Railway edge's transient
/// `400/502 "upstream error"` answers, edge error pages that fail JSON
/// parsing, transport resets) is exactly what one retry heals.
pub(crate) fn should_retry_sync_error(error: &OrigaError) -> bool {
    !matches!(error, OrigaError::SessionExpired)
        && !crate::utils::net_timeout::is_idle_timeout(error)
}

/// Runs `op`, retrying exactly once after [`SYNC_RETRY_DELAY_MS`] when the
/// first attempt fails with a retryable error (see
/// [`should_retry_sync_error`]). `delay_ms` is injectable for tests.
pub(crate) async fn with_sync_retry_delay<T, F, Fut>(
    delay_ms: u32,
    mut op: F,
) -> Result<T, OrigaError>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = Result<T, OrigaError>>,
{
    match op().await {
        Ok(value) => Ok(value),
        Err(first) if should_retry_sync_error(&first) => {
            tracing::debug!("Sync attempt failed once; retrying after delay");
            retry_delay(delay_ms).await;
            op().await
        },
        Err(first) => Err(first),
    }
}

/// Async sleep for the retry backoff. `gloo_timers` drives the WASM
/// production path; native builds (the test harness) block the thread —
/// acceptable for single-threaded tests, which pass a zero delay.
async fn retry_delay(ms: u32) {
    #[cfg(target_arch = "wasm32")]
    {
        gloo_timers::future::TimeoutFuture::new(ms).await;
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        std::thread::sleep(std::time::Duration::from_millis(ms as u64));
    }
}

/// [`with_sync_retry_delay`] with the production delay.
pub(crate) async fn with_sync_retry<T, F, Fut>(op: F) -> Result<T, OrigaError>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = Result<T, OrigaError>>,
{
    with_sync_retry_delay(SYNC_RETRY_DELAY_MS as u32, op).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn retry_runs_the_op_a_second_time_after_a_failure() {
        let mut calls = 0;
        let result = futures::executor::block_on(with_sync_retry_delay(0, || {
            calls += 1;
            async move {
                if calls < 2 {
                    Err(OrigaError::RepositoryError {
                        reason: "transient edge hiccup".to_string(),
                    })
                } else {
                    Ok(42)
                }
            }
        }));

        assert_eq!(result.expect("retry succeeds"), 42);
        assert_eq!(calls, 2, "exactly one retry after the first failure");
    }

    #[test]
    fn persistent_failure_surfaces_after_the_single_retry() {
        let mut calls = 0;
        let result: Result<(), OrigaError> =
            futures::executor::block_on(with_sync_retry_delay(0, || {
                calls += 1;
                async move {
                    Err(OrigaError::RepositoryError {
                        reason: "still down".to_string(),
                    })
                }
            }));

        assert!(result.is_err(), "persistent failure must surface");
        assert_eq!(calls, 2, "one retry, no more");
    }

    #[test]
    fn first_attempt_success_skips_the_retry() {
        let mut calls = 0;
        let result = futures::executor::block_on(with_sync_retry_delay(0, || {
            calls += 1;
            async move { Ok("immediate") }
        }));

        assert_eq!(result.expect("success"), "immediate");
        assert_eq!(calls, 1, "no retry on success");
    }

    #[test]
    fn idle_timeout_storms_are_never_retried() {
        // ADR-053 R3-C1: the neighbour already stalled — a retry is pure
        // latency on a proven-dead route.
        let error = OrigaError::NetworkError {
            url: "https://app.example.net".to_string(),
            reason: "idle timeout after 100000 ms without data".to_string(),
        };
        assert!(!should_retry_sync_error(&error));

        let mut calls = 0;
        let result: Result<(), OrigaError> =
            futures::executor::block_on(with_sync_retry_delay(0, || {
                calls += 1;
                async { Err(error.clone()) }
            }));
        assert!(result.is_err());
        assert_eq!(calls, 1, "an idle-timeout failure must not be retried");
    }

    #[test]
    fn expired_sessions_are_not_retried() {
        // A second roundtrip cannot fix an expired session — re-login is
        // the only path, so the retry would only delay the inevitable.
        assert!(!should_retry_sync_error(&OrigaError::SessionExpired));
    }

    #[test]
    fn transport_and_edge_errors_are_retried() {
        // The Railway edge's transient failures: transport resets and its
        // HTML error pages that fail JSON parsing (mapped to
        // RepositoryError) — one retry lands after the hiccup.
        let transport = OrigaError::NetworkError {
            url: "https://app.example.net".to_string(),
            reason: "Failed to send: connection reset".to_string(),
        };
        let edge_page_parse = OrigaError::RepositoryError {
            reason: "Failed to parse response: expected value at line 1 column 1".to_string(),
        };
        assert!(should_retry_sync_error(&transport));
        assert!(should_retry_sync_error(&edge_page_parse));
    }
}
