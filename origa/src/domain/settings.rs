use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct UserSettings {
    llm: LlmSettings,
    duolingo_jwt_token: Option<String>,
    telegram_user_id: Option<u64>,
    reminders_enabled: bool,
}

impl UserSettings {
    pub fn new(llm: LlmSettings, duolingo_jwt_token: Option<String>) -> Self {
        Self {
            llm,
            duolingo_jwt_token,
            reminders_enabled: false,
            telegram_user_id: None,
        }
    }

    pub fn empty() -> Self {
        Self {
            duolingo_jwt_token: None,
            llm: LlmSettings::None,
            reminders_enabled: false,
            telegram_user_id: None,
        }
    }

    pub fn llm(&self) -> &LlmSettings {
        &self.llm
    }

    pub fn set_llm(&mut self, llm: LlmSettings) {
        self.llm = llm;
    }

    pub fn duolingo_jwt_token(&self) -> Option<&str> {
        self.duolingo_jwt_token.as_deref()
    }

    pub fn set_duolingo_jwt_token(&mut self, token: Option<String>) {
        self.duolingo_jwt_token = token;
    }

    pub fn reminders_enabled(&self) -> bool {
        self.reminders_enabled
    }

    pub fn set_reminders_enabled(&mut self, enabled: bool) {
        self.reminders_enabled = enabled;
    }

    pub fn telegram_user_id(&self) -> Option<&u64> {
        self.telegram_user_id.as_ref()
    }

    pub fn set_telegram_user_id(&mut self, telegram_user_id: Option<u64>) {
        self.telegram_user_id = telegram_user_id;
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub enum LlmSettings {
    #[default]
    None,
    Gemini {
        temperature: f32,
        model: String,
    },
    OpenAi {
        temperature: f32,
        model: String,
        base_url: String,
        env_var_name: String,
    },
}
