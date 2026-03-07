use crate::app::AuthContext;
use crate::repository::{TrailBaseClient, set_session};
use origa::domain::{NativeLanguage, User};
use origa::traits::UserRepository;

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

pub async fn handle_email_password_login(
    ctx: &AuthContext,
    email: &str,
    password: &str,
) -> Result<User, String> {
    let client = TrailBaseClient::new();
    let session = client
        .login_with_email_password(email, password)
        .await
        .map_err(|e| format!("Не удалось войти: {}", e))?;

    if session.email.is_empty() {
        return Err("Email не найден в токене авторизации".to_string());
    }

    get_or_create_profile(ctx, &session.email).await
}
