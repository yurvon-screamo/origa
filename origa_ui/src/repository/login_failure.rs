//! Login failure classification shared by the email/password and OAuth
//! login paths.
//!
//! The login UI must never show "Invalid email or password" for a
//! transport failure — App Review (2026-09) saw exactly that masking when
//! an idle timeout hit the login endpoint with correct credentials
//! entered. The classifier is pure so native tests can pin the mapping.

use super::trailbase_client::AuthError;

/// Why a login attempt failed, coarse enough to pick a user-facing message.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoginFailure {
    /// The server answered 401: the credentials are actually wrong.
    InvalidCredentials,
    /// Retryable failure where the credentials are not implicated: a
    /// transport fault (timeout, DNS, offline), a server-side outage
    /// (5xx, 429) — or, as an accepted compromise, a rare local storage
    /// fault losing the just-established session ("session not found").
    /// The retry hint remains the correct user action in all three cases.
    Network,
    /// The session was established but the profile bootstrap afterwards
    /// failed; the user is signed in and a retry resumes cleanly.
    ProfileSync,
}

/// Classifies an auth-transport error for the login UI. Only the explicit
/// 401 variant may surface as invalid credentials — everything else
/// (timeouts, DNS, 5xx, parse failures) falls into the retry bucket.
pub fn classify_login_failure(error: &AuthError) -> LoginFailure {
    match error {
        AuthError::InvalidCredentials => LoginFailure::InvalidCredentials,
        AuthError::SessionExpired
        | AuthError::NetworkError(_)
        | AuthError::ServerError(_)
        | AuthError::ApiError(_) => LoginFailure::Network,
    }
}

/// Outcome of the OAuth session-establishment path: either a transport
/// failure (retryable — the UI shows the retry hint) or an already
/// rendered localized message for the OAuth alert.
#[derive(Debug)]
pub enum OAuthFailure {
    Network,
    Message(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case(AuthError::InvalidCredentials, LoginFailure::InvalidCredentials)]
    #[case(AuthError::SessionExpired, LoginFailure::Network)]
    #[case(
        AuthError::NetworkError("idle timeout after 10000 ms without data".to_string()),
        LoginFailure::Network
    )]
    #[case(
        AuthError::ServerError("Login failed: Bad Gateway".to_string()),
        LoginFailure::Network
    )]
    #[case(
        AuthError::ApiError("Login failed: Internal Server Error".to_string()),
        LoginFailure::Network
    )]
    fn auth_errors_classify_into_login_failures(
        #[case] error: AuthError,
        #[case] expected: LoginFailure,
    ) {
        assert_eq!(classify_login_failure(&error), expected);
    }
}
