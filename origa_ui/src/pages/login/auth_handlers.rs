use crate::repository::{set_session, TrailBaseClient};
use crate::store::auth_store::AuthStore;
use gloo_storage::{LocalStorage, Storage};
use origa::domain::{NativeLanguage, User};
use origa::traits::UserRepository;

pub async fn get_or_create_profile(auth_store: &AuthStore, email: &str) -> Result<User, String> {
    auth_store
        .repository()
        .merge_current_user()
        .await
        .map_err(|e| format!("Не удалось синхронизировать профиль: {}", e))?;

    match auth_store.repository().get_current_user().await {
        Ok(Some(user)) => Ok(user),
        Ok(None) => {
            let new_user = User::new(email.to_string(), NativeLanguage::Russian, None);

            auth_store
                .repository()
                .save(&new_user)
                .await
                .map_err(|e| format!("Не удалось создать профиль: {}", e))?;

            auth_store
                .repository()
                .get_current_user()
                .await
                .map_err(|e| format!("Не удалось загрузить профиль: {}", e))?
                .ok_or_else(|| "Профиль не найден после создания".to_string())
        },
        Err(e) => Err(format!("Не удалось загрузить профиль: {}", e)),
    }
}

pub async fn handle_oauth_callback(
    url_fragment: &str,
    auth_store: &AuthStore,
) -> Result<User, String> {
    let session = TrailBaseClient::parse_tokens_from_url(url_fragment)?;
    set_session(&session).map_err(|e| format!("Не удалось сохранить сессию: {}", e))?;

    if session.email.is_empty() {
        return Err("Email не найден в токене авторизации. Попробуйте войти снова.".to_string());
    }

    get_or_create_profile(auth_store, &session.email).await
}

pub async fn handle_oauth_callback_desktop(
    url: &str,
    auth_store: &AuthStore,
) -> Result<User, String> {
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

    get_or_create_profile(auth_store, &session.email).await
}
