use crate::repository::api_response::ApiResponse;
use crate::repository::trailbase_client::{AuthError, AuthRequestClient};
use gloo_net::http::Method;
use serde::{Deserialize, Serialize, de::DeserializeOwned};

#[derive(Clone)]
pub struct RecordApi<C: AuthRequestClient> {
    client: C,
    table_name: String,
}

impl<C: AuthRequestClient> RecordApi<C> {
    pub fn new(client: C, table_name: String) -> Self {
        Self { client, table_name }
    }

    pub async fn list_filtered<T: DeserializeOwned>(
        &self,
        column: &str,
        value: &str,
    ) -> Result<Vec<T>, AuthError> {
        let path = format!(
            "/api/records/v1/{}?filter[{}][$eq]={}",
            self.table_name,
            urlencoding::encode(column),
            urlencoding::encode(value)
        );
        let response = self
            .client
            .request_with_auth(&path, Method::GET, None::<&()>)
            .await?;

        if !response.ok() {
            return Err(classified_error("Failed to fetch records", &response));
        }

        #[derive(Deserialize)]
        struct ListResponseInner<T> {
            records: Vec<T>,
        }

        let list: ListResponseInner<T> = response
            .json()
            .map_err(|e| AuthError::ApiError(format!("Failed to parse response: {}", e)))?;
        Ok(list.records)
    }

    pub async fn create<T: Serialize + std::fmt::Debug>(
        &self,
        record: &T,
    ) -> Result<String, AuthError> {
        let path = format!("/api/records/v1/{}", self.table_name);
        let response = self
            .client
            .request_with_auth(&path, Method::POST, Some(record))
            .await?;

        if !response.ok() {
            let error = classified_error("Failed to create record", &response);
            tracing::error!(
                table = %self.table_name,
                status = response.status(),
                error = %error,
                "Failed to create record"
            );
            return Err(error);
        }

        #[derive(Deserialize)]
        struct CreateResponse {
            ids: Vec<String>,
        }

        let create_response: CreateResponse = response
            .json()
            .map_err(|e| AuthError::ApiError(format!("Failed to parse response: {}", e)))?;
        create_response
            .ids
            .first()
            .cloned()
            .ok_or_else(|| AuthError::ApiError("No ID returned".to_string()))
    }

    pub async fn update<T: Serialize>(&self, id: &str, record: &T) -> Result<(), AuthError> {
        let path = format!("/api/records/v1/{}/{}", self.table_name, id);
        let response = self
            .client
            .request_with_auth(&path, Method::PATCH, Some(record))
            .await?;

        if !response.ok() {
            let error = classified_error("Failed to update record", &response);
            tracing::error!(
                table = %self.table_name,
                status = response.status(),
                error = %error,
                "Failed to update record"
            );
            return Err(error);
        }

        Ok(())
    }

    pub async fn delete(&self, id: &str) -> Result<(), AuthError> {
        let path = format!("/api/records/v1/{}/{}", self.table_name, id);
        let response = self
            .client
            .request_with_auth::<()>(&path, Method::DELETE, None)
            .await?;

        if !response.ok() {
            let error = classified_error("Failed to delete record", &response);
            tracing::error!(
                table = %self.table_name,
                status = response.status(),
                error = %error,
                "Failed to delete record"
            );
            return Err(error);
        }

        Ok(())
    }
}

/// Classifies a non-2xx answer by STATUS, before anything touches the
/// body. The transport chain in front of TrailBase (VPS Caddy → Railway
/// edge) intermittently breaks requests — the server HTTP logs show it
/// as `GET /api/records/v1/domain_user 499 1ms` flaps — and the browser
/// then receives an empty or HTML error page. Parsing such a body used
/// to surface as the cryptic `Failed to parse response: expected value
/// at line 1 column 1` and killed Yandex registrations (2026-09-20).
///
/// - 401: the transport's internal refresh-and-retry already ran, so the
///   session is genuinely dead — [`AuthError::SessionExpired`] stops the
///   sync retry gate from wasting a roundtrip (re-login is the only path).
/// - 5xx / 429: retryable server-side failure — [`AuthError::ServerError`].
/// - everything else: [`AuthError::ApiError`] carrying the status and a
///   short body/status snippet.
fn classified_error(action: &str, response: &ApiResponse) -> AuthError {
    let status = response.status();
    let mut snippet = extract_error_text(response);
    if snippet.trim().is_empty() {
        snippet = response.status_text();
    }
    // Cap the snippet: proxy error pages can be a full HTML document.
    let snippet: String = snippet.trim().chars().take(200).collect();
    let detail = if snippet.is_empty() {
        format!("HTTP {status}")
    } else {
        format!("HTTP {status}: {snippet}")
    };

    let message = format!("{action}: {detail}");
    if status == 401 {
        AuthError::SessionExpired
    } else if status >= 500 || status == 429 {
        AuthError::ServerError(message)
    } else {
        AuthError::ApiError(message)
    }
}

pub fn extract_error_text(response: &ApiResponse) -> String {
    response
        .text()
        .unwrap_or_else(|_| "Unknown error".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Canned-transport stand-in: answers every request with the same
    /// pre-programmed status/body. Lets the native tests drive the record
    /// API without a browser fetch.
    #[derive(Clone)]
    struct MockClient {
        status: u16,
        status_text: &'static str,
        body: Vec<u8>,
    }

    impl AuthRequestClient for MockClient {
        async fn request_with_auth<T: Serialize>(
            &self,
            _path: &str,
            _method: Method,
            _body: Option<&T>,
        ) -> Result<ApiResponse, AuthError> {
            Ok(ApiResponse::new(
                self.status,
                self.status_text,
                self.body.clone(),
            ))
        }
    }

    fn mock_api(status: u16, status_text: &'static str, body: &[u8]) -> RecordApi<MockClient> {
        RecordApi::new(
            MockClient {
                status,
                status_text,
                body: body.to_vec(),
            },
            "domain_user".to_string(),
        )
    }

    /// The Railway edge's documented transient answer (see
    /// `should_retry_sync_error`): a `502 "upstream error"` page. Parsing
    /// it is what produced the exact reported string —
    /// `API error: Failed to parse response: Repository error: expected
    /// value at line 1 column 1`.
    const EDGE_ERROR_PAGE: &[u8] = b"upstream error\n";

    /// THE PRODUCTION REPRODUCTION (Yandex registration failure,
    /// reported 2026-09-20): the VPS-Caddy↔Railway hop intermittently
    /// breaks a request (visible as `GET /api/records/v1/domain_user 499
    /// 1ms` in the server HTTP logs) and the browser receives an
    /// empty-body error answer. The list path must classify that by
    /// STATUS — retryable server error — instead of trying to JSON-parse
    /// the emptiness and failing with the cryptic
    /// `Failed to parse response: expected value at line 1 column 1`,
    /// which is what killed the registration with "Не удалось создать
    /// профиль: …".
    #[test]
    fn empty_5xx_answer_is_a_retryable_server_error_not_a_parse_error() {
        let api = mock_api(502, "", EDGE_ERROR_PAGE);

        let result: Result<Vec<serde_json::Value>, AuthError> =
            futures::executor::block_on(api.list_filtered("email", "user@yandex.ru"));

        let error = result.expect_err("a 502 must not parse into records");
        let rendered = error.to_string();
        assert!(
            !rendered.contains("Failed to parse response"),
            "the status must be classified before body parsing, got: {rendered}"
        );
        assert!(
            !rendered.contains("expected value at line 1 column 1"),
            "the raw serde internals must never surface, got: {rendered}"
        );
        assert!(
            matches!(error, AuthError::ServerError(ref message) if message.contains("502")),
            "an empty 502 is a retryable server failure carrying the status, got: {rendered}"
        );
    }

    /// Same failure class through a real TrailBase status: ACL denials and
    /// auth rejections answer with an EMPTY body (observed live:
    /// `GET /api/records/v1/domain_user 403` with `content-length: 0`).
    #[test]
    fn empty_4xx_answer_carries_the_status_not_the_parse_noise() {
        let api = mock_api(403, "", &[]);

        let result: Result<Vec<serde_json::Value>, AuthError> =
            futures::executor::block_on(api.list_filtered("email", "user@yandex.ru"));

        let error = result.expect_err("a 403 must not parse into records");
        let rendered = error.to_string();
        assert!(
            !rendered.contains("expected value at line 1 column 1"),
            "got: {rendered}"
        );
        assert!(
            matches!(error, AuthError::ApiError(ref message) if message.contains("403")),
            "got: {rendered}"
        );
    }

    /// A 401 that survived the transport's internal refresh-and-retry
    /// means the session is dead: SessionExpired (which the sync retry
    /// gate refuses to re-run — re-login is the only path).
    #[test]
    fn unauthorized_answer_maps_to_session_expired() {
        let api = mock_api(401, "", &[]);

        let result: Result<Vec<serde_json::Value>, AuthError> =
            futures::executor::block_on(api.list_filtered("email", "user@yandex.ru"));

        assert!(matches!(result, Err(AuthError::SessionExpired)));
    }

    /// The guard rail for the classification change: a healthy answer
    /// still parses into its records.
    #[test]
    fn healthy_list_answer_still_parses() {
        let api = mock_api(200, "OK", br#"{"records":[{"id":7}]}"#);

        let result: Result<Vec<serde_json::Value>, AuthError> =
            futures::executor::block_on(api.list_filtered("email", "user@yandex.ru"));

        let records = result.expect("a healthy answer parses");
        assert_eq!(records.len(), 1);
        assert_eq!(records[0]["id"], 7);
    }

    /// The create endpoint sits on the same registration path; its
    /// non-2xx answers must carry the status too (an empty 502 body
    /// renders the old message as a bare "Failed to create record: ").
    #[test]
    fn create_failure_carries_the_status_not_an_empty_text() {
        let api = mock_api(502, "", &[]);

        let result = futures::executor::block_on(api.create(&serde_json::json!({"x": 1})));

        let error = result.expect_err("a 502 must fail the create");
        assert!(
            matches!(error, AuthError::ServerError(ref message) if message.contains("502")),
            "got: {error}"
        );
    }
}
