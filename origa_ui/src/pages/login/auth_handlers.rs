use crate::i18n::{I18nContext, Locale};
use crate::repository::{TrailBaseClient, set_session};
use crate::store::auth_store::AuthStore;
use gloo_storage::{LocalStorage, Storage};
use origa::domain::{NativeLanguage, User};
use origa::traits::UserRepository;

pub async fn get_or_create_profile(
    auth_store: &AuthStore,
    email: &str,
    i18n: &I18nContext<Locale>,
) -> Result<User, String> {
    auth_store
        .repository()
        .merge_current_user()
        .await
        .map_err(|e| {
            i18n.get_keys()
                .login()
                .sync_profile_error()
                .inner()
                .replace("{}", &e.to_string())
        })?;

    match auth_store.repository().get_current_user().await {
        Ok(Some(user)) => Ok(user),
        Ok(None) => {
            let new_user = User::new(email.to_string(), NativeLanguage::Russian, None);

            auth_store.repository().save(&new_user).await.map_err(|e| {
                i18n.get_keys()
                    .login()
                    .create_profile_error()
                    .inner()
                    .replace("{}", &e.to_string())
            })?;

            auth_store
                .repository()
                .get_current_user()
                .await
                .map_err(|e| {
                    i18n.get_keys()
                        .login()
                        .load_profile_error()
                        .inner()
                        .replace("{}", &e.to_string())
                })?
                .ok_or_else(|| {
                    i18n.get_keys()
                        .login()
                        .profile_not_found()
                        .inner()
                        .to_string()
                })
        },
        Err(e) => Err(i18n
            .get_keys()
            .login()
            .load_profile_error()
            .inner()
            .replace("{}", &e.to_string())),
    }
}

pub async fn handle_oauth_callback(
    url_fragment: &str,
    auth_store: &AuthStore,
    i18n: &I18nContext<Locale>,
) -> Result<User, String> {
    let session = TrailBaseClient::parse_tokens_from_url(url_fragment)?;
    set_session(&session).map_err(|e| {
        i18n.get_keys()
            .login()
            .save_session_error()
            .inner()
            .replace("{}", &e.to_string())
    })?;

    if session.email.is_empty() {
        return Err(i18n
            .get_keys()
            .login()
            .email_not_in_token()
            .inner()
            .to_string());
    }

    get_or_create_profile(auth_store, &session.email, i18n).await
}

pub async fn handle_oauth_callback_desktop(
    url: &str,
    auth_store: &AuthStore,
    i18n: &I18nContext<Locale>,
) -> Result<User, String> {
    let parsed = url::Url::parse(url).map_err(|e| format!("Invalid URL: {}", e))?;

    let code = parsed
        .query_pairs()
        .find(|(k, _)| k == "code")
        .map(|(_, v)| v.to_string())
        .ok_or_else(|| {
            i18n.get_keys()
                .login()
                .auth_code_not_found()
                .inner()
                .to_string()
        })?;

    let verifier: Option<String> = LocalStorage::get("pkce_verifier").ok();
    LocalStorage::delete("pkce_verifier");

    let verifier =
        verifier.ok_or_else(|| i18n.get_keys().login().pkce_not_found().inner().to_string())?;

    let client = TrailBaseClient::new();
    let session = client
        .exchange_auth_code_for_session(&code, &verifier)
        .await
        .map_err(|e| {
            i18n.get_keys()
                .login()
                .token_exchange_error()
                .inner()
                .replace("{}", &e.to_string())
        })?;

    set_session(&session).map_err(|e| {
        i18n.get_keys()
            .login()
            .save_session_error()
            .inner()
            .replace("{}", &e.to_string())
    })?;

    if session.email.is_empty() {
        return Err(i18n
            .get_keys()
            .login()
            .email_not_in_token()
            .inner()
            .to_string());
    }

    get_or_create_profile(auth_store, &session.email, i18n).await
}
