use crate::app::AuthContext;
use crate::repository::{TrailBaseClient, set_session};
use gloo_storage::{LocalStorage, Storage};
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

pub async fn handle_oauth_callback_desktop(url: &str, ctx: &AuthContext) -> Result<User, String> {
    let parsed = url::Url::parse(url).map_err(|e| format!("Неверный URL: {}", e))?;

    let code = parsed
        .query_pairs()
        .find(|(k, _)| k == "code")
        .map(|(_, v)| v.to_string())
        .ok_or("Код авторизации не найден в callback URL")?;

    let verifier: Option<String> = LocalStorage::get("pkce_verifier").ok();
    LocalStorage::delete("pkce_verifier");

    let verifier = verifier.ok_or_else(|| {
        "PKCE verifier не найден. Пожалуйста, попробуйте войти снова.".to_string()
    })?;

    let client = TrailBaseClient::new();
    let session = client
        .exchange_auth_code_for_session(&code, &verifier)
        .await
        .map_err(|e| format!("Ошибка обмена токена: {}", e))?;

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
