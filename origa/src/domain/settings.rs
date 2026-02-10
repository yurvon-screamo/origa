use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct UserSettings {
    duolingo_jwt_token: Option<String>,
    telegram_user_id: Option<u64>,
    reminders_enabled: bool,
}

impl UserSettings {
    pub fn new(duolingo_jwt_token: Option<String>) -> Self {
        Self {
            duolingo_jwt_token,
            reminders_enabled: false,
            telegram_user_id: None,
        }
    }

    pub fn empty() -> Self {
        Self {
            duolingo_jwt_token: None,
            reminders_enabled: false,
            telegram_user_id: None,
        }
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
