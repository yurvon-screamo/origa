//! API response envelope for the TrailBase transport (ADR-052, slice 1b).
//!
//! The transport reads the whole body under an idle deadline (gloo_net
//! exposes no AbortSignal, so the request goes through
//! [`crate::utils::net_timeout`]); this envelope hands the already-read
//! bytes to the call sites with the accessor surface they used on
//! `gloo_net::http::Response` — minus the async: the body is in memory.

use origa::domain::OrigaError;
use serde::de::DeserializeOwned;

pub struct ApiResponse {
    status: u16,
    status_text: String,
    bytes: Vec<u8>,
}

impl ApiResponse {
    pub fn new(status: u16, status_text: impl Into<String>, bytes: Vec<u8>) -> Self {
        Self {
            status,
            status_text: status_text.into(),
            bytes,
        }
    }

    pub fn ok(&self) -> bool {
        (200..300).contains(&self.status)
    }

    pub fn status(&self) -> u16 {
        self.status
    }

    pub fn status_text(&self) -> String {
        self.status_text.clone()
    }

    /// Decodes the body as UTF-8 text.
    pub fn text(&self) -> Result<String, OrigaError> {
        String::from_utf8(self.bytes.clone()).map_err(|e| OrigaError::RepositoryError {
            reason: e.to_string(),
        })
    }

    /// Deserializes the body as JSON.
    pub fn json<T: DeserializeOwned>(&self) -> Result<T, OrigaError> {
        serde_json::from_slice(&self.bytes).map_err(|e| OrigaError::RepositoryError {
            reason: e.to_string(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct Payload {
        token: String,
    }

    #[test]
    fn ok_reflects_the_status_class() {
        assert!(ApiResponse::new(200, "OK", Vec::new()).ok());
        assert!(ApiResponse::new(204, "No Content", Vec::new()).ok());
        assert!(!ApiResponse::new(401, "Unauthorized", Vec::new()).ok());
        assert!(!ApiResponse::new(424, "Failed Dependency", Vec::new()).ok());
        assert!(!ApiResponse::new(503, "Service Unavailable", Vec::new()).ok());
    }

    #[test]
    fn accessors_expose_the_envelope() {
        let response = ApiResponse::new(418, "I'm a teapot", Vec::new());
        assert_eq!(response.status(), 418);
        assert_eq!(response.status_text(), "I'm a teapot");
    }

    #[test]
    fn text_decodes_the_body() {
        let response = ApiResponse::new(200, "OK", b"csrf-token".to_vec());
        assert_eq!(response.text().expect("utf-8 body"), "csrf-token");
    }

    #[test]
    fn json_deserializes_the_body() {
        let response = ApiResponse::new(200, "OK", br#"{"token":"abc"}"#.to_vec());
        let payload: Payload = response.json().expect("valid json body");
        assert_eq!(
            payload,
            Payload {
                token: "abc".to_string()
            }
        );
    }

    #[test]
    fn malformed_bodies_surface_as_errors() {
        let response = ApiResponse::new(200, "OK", vec![0xff, 0xfe]);
        assert!(response.text().is_err());

        let response = ApiResponse::new(200, "OK", b"not json".to_vec());
        let result: Result<Payload, _> = response.json();
        assert!(result.is_err());
    }
}
