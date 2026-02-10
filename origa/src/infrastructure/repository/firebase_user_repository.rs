use crate::application::UserRepository;
use crate::domain::{OrigaError, User};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;
use ulid::Ulid;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct FirestoreDocument {
    name: String,
    fields: HashMap<String, FirestoreValue>,
    #[serde(rename = "createTime")]
    create_time: Option<String>,
    #[serde(rename = "updateTime")]
    update_time: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
enum FirestoreValue {
    StringValue {
        #[serde(rename = "stringValue")]
        string_value: String,
    },
    IntegerValue {
        #[serde(rename = "integerValue")]
        integer_value: String,
    },
    BooleanValue {
        #[serde(rename = "booleanValue")]
        boolean_value: bool,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct FirestoreListResponse {
    documents: Option<Vec<FirestoreDocument>>,
    #[serde(rename = "nextPageToken")]
    next_page_token: Option<String>,
}

pub struct FirebaseUserRepository {
    project_id: String,
    database_id: String,
    collection_name: String,
    access_token: String,
    client: reqwest::Client,
}

impl FirebaseUserRepository {
    pub async fn new(
        project_id: String,
        database_id: Option<String>,
        access_token: String,
    ) -> Result<Self, OrigaError> {
        let client =
            reqwest::Client::builder()
                .build()
                .map_err(|e| OrigaError::RepositoryError {
                    reason: format!("Failed to create HTTP client: {}", e),
                })?;

        Ok(Self {
            project_id,
            database_id: database_id.unwrap_or_else(|| "(default)".to_string()),
            collection_name: "users".to_string(),
            access_token,
            client,
        })
    }

    pub fn with_collection_name(mut self, collection_name: String) -> Self {
        self.collection_name = collection_name;
        self
    }

    fn base_url(&self) -> String {
        format!(
            "https://firestore.googleapis.com/v1/projects/{}/databases/{}/documents",
            self.project_id, self.database_id
        )
    }

    fn document_url(&self, user_id: Ulid) -> String {
        format!("{}/{}/{}", self.base_url(), self.collection_name, user_id)
    }

    fn collection_url(&self) -> String {
        format!("{}/{}", self.base_url(), self.collection_name)
    }

    fn user_to_firestore_document(&self, user: &User) -> Result<FirestoreDocument, OrigaError> {
        let json_str = serde_json::to_string(user).map_err(|e| OrigaError::RepositoryError {
            reason: format!("Failed to serialize user to JSON: {}", e),
        })?;

        let mut fields = HashMap::new();
        fields.insert(
            "data".to_string(),
            FirestoreValue::StringValue {
                string_value: json_str,
            },
        );

        Ok(FirestoreDocument {
            name: format!(
                "projects/{}/databases/{}/documents/{}/{}",
                self.project_id,
                self.database_id,
                self.collection_name,
                user.id()
            ),
            fields,
            create_time: None,
            update_time: None,
        })
    }

    fn firestore_document_to_user(&self, doc: FirestoreDocument) -> Result<User, OrigaError> {
        let data_field = doc
            .fields
            .get("data")
            .ok_or_else(|| OrigaError::RepositoryError {
                reason: "Document missing 'data' field".to_string(),
            })?;

        let json_str = match data_field {
            FirestoreValue::StringValue { string_value } => string_value,
            _ => {
                return Err(OrigaError::RepositoryError {
                    reason: "Data field is not a string".to_string(),
                });
            }
        };

        serde_json::from_str(json_str).map_err(|e| OrigaError::RepositoryError {
            reason: format!("Failed to deserialize user from JSON: {}", e),
        })
    }

    async fn make_authenticated_request<T>(
        &self,
        request: reqwest::RequestBuilder,
    ) -> Result<T, OrigaError>
    where
        T: for<'de> Deserialize<'de>,
    {
        let response = request
            .header("Authorization", format!("Bearer {}", self.access_token))
            .header("Content-Type", "application/json")
            .send()
            .await
            .map_err(|e| OrigaError::RepositoryError {
                reason: format!("HTTP request failed: {}", e),
            })?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(OrigaError::RepositoryError {
                reason: format!("Firebase API error {}: {}", status, error_text),
            });
        }

        response
            .json()
            .await
            .map_err(|e| OrigaError::RepositoryError {
                reason: format!("Failed to parse JSON response: {}", e),
            })
    }
}

#[async_trait]
impl UserRepository for FirebaseUserRepository {
    async fn list(&self) -> Result<Vec<User>, OrigaError> {
        let url = self.collection_url();
        let request = self.client.get(&url).timeout(Duration::from_secs(30));

        let response: FirestoreListResponse = self.make_authenticated_request(request).await?;

        let documents = response.documents.unwrap_or_default();
        let mut users = Vec::with_capacity(documents.len());

        for doc in documents {
            let user = self.firestore_document_to_user(doc)?;
            users.push(user);
        }

        Ok(users)
    }

    async fn find_by_telegram_id(&self, telegram_id: &u64) -> Result<Option<User>, OrigaError> {
        let users = self.list().await?;

        let user = users
            .into_iter()
            .find(|x| x.settings().telegram_user_id() == Some(telegram_id));

        Ok(user)
    }

    async fn find_by_id(&self, user_id: Ulid) -> Result<Option<User>, OrigaError> {
        let url = self.document_url(user_id);
        let request = self.client.get(&url);

        let response = request
            .header("Authorization", format!("Bearer {}", self.access_token))
            .header("Content-Type", "application/json")
            .send()
            .await
            .map_err(|e| OrigaError::RepositoryError {
                reason: format!("HTTP request failed: {}", e),
            })?;

        match response.status() {
            reqwest::StatusCode::OK => {
                let doc: FirestoreDocument =
                    response
                        .json()
                        .await
                        .map_err(|e| OrigaError::RepositoryError {
                            reason: format!("Failed to parse JSON response: {}", e),
                        })?;

                let user = self.firestore_document_to_user(doc)?;
                Ok(Some(user))
            }
            reqwest::StatusCode::NOT_FOUND => Ok(None),
            status => {
                let error_text = response
                    .text()
                    .await
                    .unwrap_or_else(|_| "Unknown error".to_string());
                Err(OrigaError::RepositoryError {
                    reason: format!("Firebase API error {}: {}", status, error_text),
                })
            }
        }
    }

    async fn save(&self, user: &User) -> Result<(), OrigaError> {
        let url = self.document_url(user.id());
        let document = self.user_to_firestore_document(user)?;

        let request = self
            .client
            .patch(&url)
            .json(&document)
            .timeout(Duration::from_secs(30));

        let response: FirestoreDocument = self.make_authenticated_request(request).await?;
        dbg!(response);

        Ok(())
    }

    async fn delete(&self, user_id: Ulid) -> Result<(), OrigaError> {
        let url = self.document_url(user_id);
        let request = self.client.delete(&url).timeout(Duration::from_secs(30));

        let response = request
            .header("Authorization", format!("Bearer {}", self.access_token))
            .send()
            .await
            .map_err(|e| OrigaError::RepositoryError {
                reason: format!("HTTP request failed: {}", e),
            })?;

        match response.status() {
            reqwest::StatusCode::OK | reqwest::StatusCode::NOT_FOUND => Ok(()),
            status => {
                let error_text = response
                    .text()
                    .await
                    .unwrap_or_else(|_| "Unknown error".to_string());
                Err(OrigaError::RepositoryError {
                    reason: format!("Firebase API error {}: {}", status, error_text),
                })
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{JapaneseLevel, NativeLanguage};

    #[tokio::test]
    async fn test_user_serialization() {
        let user = User::new(
            "testuser".to_string(),
            JapaneseLevel::N5,
            NativeLanguage::English,
        );

        let repo =
            FirebaseUserRepository::new("test-project".to_string(), None, "fake-token".to_string())
                .await
                .unwrap();

        let doc = repo.user_to_firestore_document(&user).unwrap();
        let restored_user = repo.firestore_document_to_user(doc).unwrap();

        assert_eq!(user.id(), restored_user.id());
        assert_eq!(user.username(), restored_user.username());
        assert_eq!(
            user.current_japanese_level(),
            restored_user.current_japanese_level()
        );
        assert_eq!(user.native_language(), restored_user.native_language());
    }

    #[tokio::test]
    async fn test_repository_creation() {
        let repo = FirebaseUserRepository::new(
            "my-project".to_string(),
            Some("my-database".to_string()),
            "access-token".to_string(),
        )
        .await
        .unwrap()
        .with_collection_name("my-users".to_string());

        assert_eq!(repo.project_id, "my-project");
        assert_eq!(repo.database_id, "my-database");
        assert_eq!(repo.collection_name, "my-users");
    }

    #[tokio::test]
    async fn test_url_generation() {
        let repo =
            FirebaseUserRepository::new("test-project".to_string(), None, "token".to_string())
                .await
                .unwrap();

        let user_id = Ulid::new();
        let expected_base = "https://firestore.googleapis.com/v1/projects/test-project/databases/(default)/documents";

        assert_eq!(repo.base_url(), expected_base);
        assert_eq!(repo.collection_url(), format!("{}/users", expected_base));
        assert_eq!(
            repo.document_url(user_id),
            format!("{}/users/{}", expected_base, user_id)
        );
    }
}
