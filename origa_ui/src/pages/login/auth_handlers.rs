use crate::app::AuthContext;
use crate::repository::{OAuthProvider, TrailBaseClient, set_session};
use origa::application::UserRepository;
use origa::domain::{NativeLanguage, User};

pub async fn get_or_create_profile(ctx: &AuthContext, email: &str) -> Result<User, String> {
    match ctx.repository.find_by_email(email).await {
        Ok(Some(user)) => Ok(user),
        Ok(None) => {
            let new_user = User::new(email.to_string(), NativeLanguage::Russian, None);

            ctx.repository
                .save(&new_user)
                .await
                .map_err(|e| format!("Не удалось создать профиль: {}", e))?;

            ctx.repository
                .find_by_email(email)
                .await
                .map_err(|e| format!("Не удалось загрузить профиль: {}", e))?
                .ok_or_else(|| "Профиль не найден после создания".to_string())
        }
        Err(e) => Err(format!("Не удалось загрузить профиль: {}", e)),
    }
}

pub async fn handle_oauth_callback(url_fragment: &str, ctx: &AuthContext) -> Result<User, String> {
    let session = TrailBaseClient::parse_tokens_from_url(url_fragment)?;
    set_session(&session).map_err(|e| format!("Не удалось сохранить сессию: {}", e))?;

    if session.email.is_empty() {
        return Err("Email не найден в токене авторизации. Попробуйте войти снова.".to_string());
    }

    get_or_create_profile(ctx, &session.email).await
}

pub fn handle_oauth_login(provider: OAuthProvider) {
    let redirect_uri = "origa://auth/callback";
    let url = TrailBaseClient::get_oauth_url(provider.as_str(), redirect_uri);

    if let Some(window) = web_sys::window() {
        let _ = window.location().set_href(&url);
    }
}
