use std::sync::{Arc, LazyLock};

use crate::domain::OrigaError;
use crate::infrastructure::{FsrsSrsService, LlmServiceInvoker, OpenAiLlm};
use serde::{Deserialize, Serialize};
use tokio::sync::OnceCell;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub enum LlmSettings {
    #[default]
    None,
    OpenAi {
        temperature: f32,
        model: String,
        base_url: String,
        env_var_name: String,
    },
}

static SETTINGS: LazyLock<ApplicationEnvironment> = LazyLock::new(|| ApplicationEnvironment {
    lazy_srs_service: Arc::new(OnceCell::new()),
});

pub struct ApplicationEnvironment {
    lazy_srs_service: Arc<OnceCell<FsrsSrsService>>,
}

impl ApplicationEnvironment {
    pub async fn get_llm_service(
        &self,
        llm_settings: &LlmSettings,
    ) -> Result<LlmServiceInvoker, OrigaError> {
        let service = match llm_settings {
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
