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
        self.list_by_path(&path).await
    }

    /// Lists records by a fully-built filter path (used by the delta
    /// probe, which targets one row by id + updated_at).
    pub async fn list_by_path<T: DeserializeOwned>(&self, path: &str) -> Result<Vec<T>, AuthError> {
        let response = self
            .client
            .request_with_auth(path, Method::GET, None::<&()>)
            .await?;

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
            let error_text = extract_error_text(&response);
            tracing::error!(
                table = %self.table_name,
                status = response.status(),
                error = %error_text,
                "Failed to create record"
            );
            return Err(AuthError::ApiError(format!(
                "Failed to create record: {}",
                error_text
            )));
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
            let error_text = extract_error_text(&response);
            tracing::error!(
                table = %self.table_name,
                status = response.status(),
                error = %error_text,
                "Failed to update record"
            );
            return Err(AuthError::ApiError(format!(
                "Failed to update record: {}",
                error_text
            )));
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
            let error_text = extract_error_text(&response);
            tracing::error!(
                table = %self.table_name,
                status = response.status(),
                error = %error_text,
                "Failed to delete record"
            );
            return Err(AuthError::ApiError(format!(
                "Failed to delete record: {}",
                error_text
            )));
        }

        Ok(())
    }
}

pub fn extract_error_text(response: &ApiResponse) -> String {
    response
        .text()
        .unwrap_or_else(|_| "Unknown error".to_string())
}
