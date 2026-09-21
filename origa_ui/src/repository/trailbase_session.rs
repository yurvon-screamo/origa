use crate::repository::session::{
    TrailBaseSession, clear_session_async, get_session, set_session_async,
};
use crate::repository::trailbase_auth::decode_jwt_claims;
use crate::repository::trailbase_client::{
    AuthError, AuthRequestClient, AuthTokenResponse, TrailBaseClient,
};

use crate::repository::api_response::ApiResponse;
use gloo_net::http::Method;
use serde::Serialize;
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::atomic::{AtomicBool, Ordering};
use tracing::debug;

// Session lifecycle: refresh, logout, password change.
// HTTP transport layer: see trailbase_client.rs
//
// INVARIANT: On Tauri, the sync `get_session()` reads ONLY from the
// in-memory cache (no localStorage fallback). The cache is populated by
// `get_session_async()` during `check_session()` at app start, before
// `ProtectedRoute` renders any authenticated page. Therefore, by the time
// these methods run, the cache is guaranteed to be populated if the user
// is authenticated. See ADR-010 for details.

const REFRESH_THRESHOLD_SECONDS: u64 = 300;
const REFRESH_TIMEOUT_MS: u32 = 30000;
const REFRESH_RETRY_DELAY_MS: u32 = 100;
/// Bounded raciness handling for [`refresh_under_single_flight`]: a caller
/// that keeps losing the acquire to fresher winners adopts their session
/// instead of looping forever.
const REFRESH_ATTEMPT_LIMIT: usize = 3;

/// Single-flight gate for session refreshes. TrailBase rotates the
/// refresh token on every `/refresh`: two parallel refreshes with the same
/// token mean the loser answers 401 → `SessionExpired` → a spurious
/// logout (observed on desktop, 2026-09). Every refresh path funnels
/// through one gate: the winner runs the POST, the losers wait and adopt
/// the winner's session.
pub(crate) struct RefreshGate {
    in_progress: AtomicBool,
}

impl RefreshGate {
    pub(crate) const fn new() -> Self {
        Self {
            in_progress: AtomicBool::new(false),
        }
    }

    fn is_in_progress(&self) -> bool {
        self.in_progress.load(Ordering::SeqCst)
    }

    /// Acquires the gate; `false` when a refresh is already running.
    fn try_acquire(&self) -> bool {
        self.in_progress
            .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
            .is_ok()
    }

    fn release(&self) {
        self.in_progress.store(false, Ordering::SeqCst);
    }

    /// Polls until the in-flight refresh completes (or the deadline
    /// passes). The poll delay is cooperative on native builds — a
    /// blocking sleep would deadlock the single-threaded test executor.
    async fn wait_until_idle(&self, timeout_ms: u32) -> Result<(), AuthError> {
        let start = current_timestamp();
        let timeout_secs = timeout_ms as u64 / 1000;

        while self.is_in_progress() {
            let elapsed = current_timestamp().saturating_sub(start);
            if elapsed >= timeout_secs {
                tracing::error!(
                    waited_secs = elapsed,
                    "Session refresh coordination timeout"
                );
                return Err(AuthError::ApiError(
                    "Refresh coordination timeout".to_string(),
                ));
            }

            debug!("Waiting for refresh completion, elapsed: {}s", elapsed);
            gate_sleep(REFRESH_RETRY_DELAY_MS).await;
        }
        Ok(())
    }
}

/// The production gate instance. Native tests construct private gates so
/// the global never races across parallel test cases.
static REFRESH_GATE: RefreshGate = RefreshGate::new();

/// Backoff sleep for the gate's poll loop. WASM drives the real timer;
/// native (the test harness) yields cooperatively so `block_on` and
/// `join!` interleave without blocking the thread.
fn gate_sleep(ms: u32) -> Pin<Box<dyn Future<Output = ()>>> {
    #[cfg(target_arch = "wasm32")]
    {
        Box::pin(gloo_timers::future::TimeoutFuture::new(ms))
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = ms;
        Box::pin(yield_cooperatively())
    }
}

/// Yields once (pending with a self-scheduled wake, ready on the next
/// poll) so the single-threaded executor interleaves other tasks instead
/// of spinning or blocking the thread.
#[cfg(not(target_arch = "wasm32"))]
fn yield_cooperatively() -> impl Future<Output = ()> {
    let mut yielded = false;
    futures::future::poll_fn(move |cx| {
        if yielded {
            std::task::Poll::Ready(())
        } else {
            yielded = true;
            cx.waker().wake_by_ref();
            std::task::Poll::Pending
        }
    })
}

pub fn is_refresh_in_progress() -> bool {
    REFRESH_GATE.is_in_progress()
}

/// The single-flight refresh discipline: waits out any in-flight refresh,
/// adopts the winner's session when it moved, otherwise acquires the gate
/// and runs `refresh` exactly once per simultaneous caller set. Extracted
/// from the client so the concurrency contract runs against injected
/// closures in native tests. The op returns an owned boxed future — the
/// production op borrows the client via a clone.
async fn refresh_under_single_flight<F, R>(
    gate: &RefreshGate,
    read_session: F,
    refresh: R,
) -> Result<TrailBaseSession, AuthError>
where
    F: Fn() -> Option<TrailBaseSession>,
    R: Fn(String) -> Pin<Box<dyn Future<Output = Result<TrailBaseSession, AuthError>>>>,
{
    for _ in 0..REFRESH_ATTEMPT_LIMIT {
        gate.wait_until_idle(REFRESH_TIMEOUT_MS).await?;

        let Some(session) = read_session() else {
            return Err(AuthError::SessionExpired);
        };
        if !should_refresh_session(session.expires_at) {
            // A concurrent winner already refreshed for us.
            return Ok(session);
        }
        if session.refresh_token.is_empty() {
            return Err(AuthError::SessionExpired);
        }

        if gate.try_acquire() {
            let token = session.refresh_token.clone();
            let result = refresh(token).await;
            gate.release();
            return result;
        }
        // Lost the acquire race — loop, wait for the winner, adopt.
    }
    Err(AuthError::ApiError(
        "Refresh coordination timeout".to_string(),
    ))
}

pub fn should_refresh_session(expires_at: u64) -> bool {
    let now = current_timestamp();
    now.saturating_add(REFRESH_THRESHOLD_SECONDS) >= expires_at
}

pub fn current_timestamp() -> u64 {
    chrono::Utc::now().timestamp().max(0) as u64
}

fn build_auth_headers(auth_token: &str) -> HashMap<String, String> {
    let mut headers = HashMap::new();
    headers.insert(
        "Authorization".to_string(),
        format!("Bearer {}", auth_token),
    );
    headers
}

#[derive(serde::Serialize)]
struct Empty {}

impl TrailBaseClient {
    pub async fn refresh_session(
        &self,
        refresh_token: &str,
    ) -> Result<TrailBaseSession, AuthError> {
        #[derive(Serialize)]
        struct RefreshRequest<'a> {
            refresh_token: &'a str,
        }

        let response = self
            .fetch(
                "/api/auth/v1/refresh",
                Method::POST,
                Some(&RefreshRequest { refresh_token }),
                None,
            )
            .await?;

        if !response.ok() {
            return Err(AuthError::SessionExpired);
        }

        let token_response: AuthTokenResponse = Self::json(&response)?;
        let claims = decode_jwt_claims(&token_response.auth_token).map_err(|e| {
            tracing::error!(error = %e, "Failed to decode refreshed session JWT");
            AuthError::ApiError(format!("Failed to decode JWT: {}", e))
        })?;

        let now = current_timestamp();
        let expires_at = claims.expires_at(now.saturating_add(3600));

        let session = TrailBaseSession {
            auth_token: token_response.auth_token,
            refresh_token: token_response
                .refresh_token
                .unwrap_or_else(|| refresh_token.to_string()),
            email: claims.email.clone().unwrap_or_default(),
            trailbase_id: claims.sub.clone(),
            record_id: None,
            expires_at,
        };

        set_session_async(&session)
            .await
            .map_err(AuthError::ApiError)?;
        Ok(session)
    }

    /// Single-flight refresh for external callers (the auth bootstrap's
    /// background refresh, the request-path 401 retry): waits out any
    /// in-flight refresh, adopts its outcome when it moved, otherwise runs
    /// exactly one POST. Never races a concurrent refresh into the token
    /// rotation 401.
    pub async fn refresh_session_gated(&self) -> Result<TrailBaseSession, AuthError> {
        let client = self.clone();
        refresh_under_single_flight(&REFRESH_GATE, get_session, move |token| {
            let client = client.clone();
            Box::pin(async move { client.refresh_session(&token).await })
        })
        .await
    }

    /// The request-path entry: skips the roundtrip entirely while the
    /// session is fresh, otherwise funnels into the shared single-flight
    /// discipline (one discipline for every refresh caller — the auth
    /// bootstrap, the pre-request check, and the 401 retry).
    async fn ensure_fresh_session(
        &self,
        session: TrailBaseSession,
        label: &str,
    ) -> Result<TrailBaseSession, AuthError> {
        if !should_refresh_session(session.expires_at) {
            return Ok(session);
        }

        if session.refresh_token.is_empty() {
            return Err(AuthError::SessionExpired);
        }

        debug!("{}: refreshing session under single flight", label);
        self.refresh_session_gated().await
    }

    async fn _request_with_auth_impl<T: Serialize>(
        &self,
        path: &str,
        method: Method,
        body: Option<&T>,
    ) -> Result<ApiResponse, AuthError> {
        let session = get_session().ok_or(AuthError::SessionExpired)?;
        let session = self.ensure_fresh_session(session, "pre-request").await?;

        let headers = build_auth_headers(&session.auth_token);

        let response = self
            .fetch(path, method.clone(), body, Some(headers))
            .await?;

        if response.status() != 401 {
            return Ok(response);
        }

        let session = get_session().ok_or(AuthError::SessionExpired)?;
        let refreshed = self.ensure_fresh_session(session, "401-retry").await?;

        let headers = build_auth_headers(&refreshed.auth_token);

        self.fetch(path, method, body, Some(headers)).await
    }

    pub async fn logout(&self) -> Result<(), String> {
        if let Some(session) = get_session() {
            let headers = build_auth_headers(&session.auth_token);
            let _ = self
                .fetch(
                    "/api/auth/v1/logout",
                    Method::POST,
                    Some(&Empty {}),
                    Some(headers),
                )
                .await;
        }
        clear_session_async().await;
        Ok(())
    }

    pub async fn change_password(
        &self,
        old_password: &str,
        new_password: &str,
        new_password_repeat: &str,
    ) -> Result<(), AuthError> {
        #[derive(Serialize)]
        struct ChangePasswordRequest<'a> {
            old_password: &'a str,
            new_password: &'a str,
            new_password_repeat: &'a str,
        }

        let response = self
            .request_with_auth(
                "/api/auth/v1/change_password",
                Method::POST,
                Some(&ChangePasswordRequest {
                    old_password,
                    new_password,
                    new_password_repeat,
                }),
            )
            .await?;

        if !response.ok() {
            let error_text = response
                .text()
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(AuthError::ApiError(format!(
                "Password change failed: {}",
                error_text
            )));
        }

        Ok(())
    }
}

impl AuthRequestClient for TrailBaseClient {
    async fn request_with_auth<T: Serialize>(
        &self,
        path: &str,
        method: Method,
        body: Option<&T>,
    ) -> Result<ApiResponse, AuthError> {
        self._request_with_auth_impl(path, method, body).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};

    fn stale_session() -> TrailBaseSession {
        TrailBaseSession {
            auth_token: "stale-auth".to_string(),
            refresh_token: "stale-token".to_string(),
            email: "a@example.com".to_string(),
            trailbase_id: "00000000-0000-0000-0000-000000000001".to_string(),
            record_id: None,
            expires_at: current_timestamp(),
        }
    }

    fn fresh_session() -> TrailBaseSession {
        TrailBaseSession {
            auth_token: "fresh-auth".to_string(),
            refresh_token: "fresh-token".to_string(),
            expires_at: current_timestamp() + 3600,
            ..stale_session()
        }
    }

    /// Yields to the executor exactly once so the winning refresh parks
    /// mid-POST and the losers reach the gate's wait — mirroring the
    /// single-threaded WASM interleaving. Local copy of the native
    /// cooperative yield: the production one is `cfg(not(wasm32))` and
    /// this test module still compiles on the wasm target.
    fn yield_once() -> impl Future<Output = ()> {
        let mut yielded = false;
        futures::future::poll_fn(move |cx| {
            if yielded {
                std::task::Poll::Ready(())
            } else {
                yielded = true;
                cx.waker().wake_by_ref();
                std::task::Poll::Pending
            }
        })
    }

    /// The concurrency contract: N simultaneous callers, exactly ONE
    /// refresh POST. The losers wait and adopt the winner's session —
    /// this is what prevents the parallel-refresh token-rotation 401
    /// (spurious logout) observed on desktop.
    #[test]
    fn parallel_callers_run_exactly_one_refresh() {
        let gate = RefreshGate::new();
        let sessions = Arc::new(Mutex::new(stale_session()));
        let refreshes_started = Arc::new(Mutex::new(0usize));

        futures::executor::block_on(async {
            let mut callers = Vec::new();
            for _ in 0..3 {
                let read_sessions = Arc::clone(&sessions);
                let op_sessions = Arc::clone(&sessions);
                let started = Arc::clone(&refreshes_started);

                callers.push(refresh_under_single_flight(
                    &gate,
                    move || Some(read_sessions.lock().unwrap().clone()),
                    move |token| {
                        let started = Arc::clone(&started);
                        let op_sessions = Arc::clone(&op_sessions);
                        Box::pin(async move {
                            assert_eq!(token, "stale-token");
                            *started.lock().unwrap() += 1;
                            yield_once().await;
                            let fresh = fresh_session();
                            *op_sessions.lock().unwrap() = fresh.clone();
                            Ok(fresh)
                        })
                    },
                ));
            }

            let results = futures::future::join_all(callers).await;
            for result in results {
                let session = result.expect("every caller must resolve");
                assert_eq!(session.refresh_token, "fresh-token");
            }
        });

        assert_eq!(
            *refreshes_started.lock().unwrap(),
            1,
            "the refresh op must run exactly once for all simultaneous callers"
        );
    }

    /// A caller arriving while the gate is idle and the session already
    /// fresh short-circuits without touching the network.
    #[test]
    fn fresh_session_never_refreshes() {
        let gate = RefreshGate::new();
        let sessions = Arc::new(Mutex::new(fresh_session()));
        let refreshes_started = Arc::new(Mutex::new(0usize));

        let read_sessions = Arc::clone(&sessions);
        let started = Arc::clone(&refreshes_started);
        futures::executor::block_on(refresh_under_single_flight(
            &gate,
            move || Some(read_sessions.lock().unwrap().clone()),
            move |_token| {
                let started = Arc::clone(&started);
                Box::pin(async move {
                    *started.lock().unwrap() += 1;
                    Ok(fresh_session())
                })
            },
        ))
        .expect("short-circuit success");

        assert_eq!(*refreshes_started.lock().unwrap(), 0);
    }
}
