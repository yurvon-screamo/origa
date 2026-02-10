use std::path::PathBuf;
use std::sync::{Arc, LazyLock};

use crate::application::UserRepository;
use crate::domain::{LlmSettings, OrigaError};
use crate::infrastructure::{
    FileSystemUserRepository, FirebaseUserRepository, FsrsSrsService, GeminiLlm, LlmServiceInvoker,
    OpenAiLlm,
};
use tokio::sync::OnceCell;

static SETTINGS: LazyLock<ApplicationEnvironment> = LazyLock::new(|| ApplicationEnvironment {
    lazy_firebase_repository: Arc::new(OnceCell::new()),
    lazy_file_repository: Arc::new(OnceCell::new()),
    lazy_srs_service: Arc::new(OnceCell::new()),
});

pub struct ApplicationEnvironment {
    lazy_firebase_repository: Arc<OnceCell<FirebaseUserRepository>>,
    lazy_file_repository: Arc<OnceCell<FileSystemUserRepository>>,
    lazy_srs_service: Arc<OnceCell<FsrsSrsService>>,
}

fn expand_tilde() -> PathBuf {
    if std::env::var("ANDROID_DATA").is_ok() {
        PathBuf::from(format!("/data/data/{}/files", "net.uwuwu.origa"))
    } else {
        let home = std::env::var("HOME")
            .or_else(|_| std::env::var("USERPROFILE")) // Windows
            .unwrap_or_else(|_| "~".to_string());

        PathBuf::from(&home).join(".origa")
    }
}

impl ApplicationEnvironment {
    pub async fn get_firebase_repository(&self) -> Result<&FirebaseUserRepository, OrigaError> {
        self.lazy_firebase_repository
            .get_or_try_init(|| async {
                // TODO: Get project id, database id, and access token from environment variables
                FirebaseUserRepository::new("origa-43210".to_string(), None, "".to_string())
                    .await
                    .map_err(|e| OrigaError::SettingsError {
                        reason: e.to_string(),
                    })
            })
            .await
    }

    pub async fn get_file_repository(&self) -> Result<&FileSystemUserRepository, OrigaError> {
        self.lazy_file_repository
            .get_or_try_init(|| async {
                FileSystemUserRepository::new(expand_tilde())
                    .await
                    .map_err(|e| OrigaError::SettingsError {
                        reason: e.to_string(),
                    })
            })
            .await
    }

    pub async fn get_llm_service(
        &self,
        user_id: ulid::Ulid,
    ) -> Result<LlmServiceInvoker, OrigaError> {
        let repository = self.get_firebase_repository().await?;
        let user = repository
            .find_by_id(user_id)
            .await?
            .ok_or(OrigaError::UserNotFound { user_id })?;
        let llm_settings = user.settings().llm();

        let service = match llm_settings {
            LlmSettings::Gemini { temperature, model } => {
                LlmServiceInvoker::Gemini(GeminiLlm::new(*temperature, model.clone()).map_err(
                    |e| OrigaError::SettingsError {
                        reason: e.to_string(),
                    },
                )?)
            }
            LlmSettings::OpenAi {
                temperature,
                model,
                base_url,
                env_var_name,
            } => LlmServiceInvoker::OpenAi(
                OpenAiLlm::new(
                    *temperature,
                    model.clone(),
                    base_url.clone(),
                    env_var_name.clone(),
                )
                .map_err(|e| OrigaError::SettingsError {
                    reason: e.to_string(),
                })?,
            ),
            LlmSettings::None => LlmServiceInvoker::None,
        };
        Ok(service)
    }

    pub async fn get_srs_service(&self) -> Result<&FsrsSrsService, OrigaError> {
        self.lazy_srs_service
            .get_or_try_init(|| async {
                FsrsSrsService::new().map_err(|e| OrigaError::SettingsError {
                    reason: e.to_string(),
                })
            })
            .await
    }

    pub fn get() -> &'static ApplicationEnvironment {
        &SETTINGS
    }
}
