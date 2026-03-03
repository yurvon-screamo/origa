use super::LoginMode;
use super::validation::validate_credentials;
use crate::app::AuthContext;
use crate::repository::{OAuthProvider, SupabaseClient, set_session};
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_router::hooks::use_navigate;
use origa::application::UserRepository;
use origa::domain::{NativeLanguage, User};

pub fn is_email_not_confirmed_error(error: &str) -> bool {
    error.contains("email_not_confirmed")
}

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

pub fn handle_login(
    email: RwSignal<String>,
    password: RwSignal<String>,
    error: RwSignal<Option<String>>,
    mode: RwSignal<LoginMode>,
    is_loading: RwSignal<bool>,
) {
    let email_val = email.get();
    let password_val = password.get();

    if let Err(e) = validate_credentials(&email_val, &password_val) {
        error.set(Some(e));
        return;
    }

    error.set(None);
    is_loading.set(true);

    let ctx = use_context::<AuthContext>().expect("AuthContext not provided");
    let navigate = use_navigate();

    spawn_local(async move {
        match ctx.client.login(&email_val, &password_val).await {
            Ok(_session) => match get_or_create_profile(&ctx, &email_val).await {
                Ok(user) => {
                    ctx.current_user.set(Some(user));
                    navigate("/home", Default::default());
                }
                Err(e) => {
                    is_loading.set(false);
                    error.set(Some(e));
                }
            },
            Err(e) => {
                is_loading.set(false);
                if is_email_not_confirmed_error(&e) {
                    error.set(None);
                    mode.set(LoginMode::EmailNotConfirmed);
                } else {
                    error.set(Some(e));
                }
            }
        }
    });
}

pub fn handle_register(
    email: RwSignal<String>,
    password: RwSignal<String>,
    error: RwSignal<Option<String>>,
    mode: RwSignal<LoginMode>,
    is_loading: RwSignal<bool>,
) {
    let email_val = email.get();
    let password_val = password.get();

    if let Err(e) = validate_credentials(&email_val, &password_val) {
        error.set(Some(e));
        return;
    }

    error.set(None);
    is_loading.set(true);

    let ctx = use_context::<AuthContext>().expect("AuthContext not provided");

    spawn_local(async move {
        match ctx.client.register(&email_val, &password_val).await {
            Ok(_user) => {
                is_loading.set(false);
                mode.set(LoginMode::EmailNotConfirmed);
            }
            Err(e) => {
                is_loading.set(false);
                if is_email_not_confirmed_error(&e) {
                    mode.set(LoginMode::EmailNotConfirmed);
                } else {
                    error.set(Some(e));
                }
            }
        }
    });
}

pub async fn handle_oauth_callback(url_fragment: &str, ctx: &AuthContext) -> Result<User, String> {
    let session = SupabaseClient::parse_tokens_from_url(url_fragment)?;
    set_session(&session).map_err(|e| format!("Не удалось сохранить сессию: {}", e))?;
    get_or_create_profile(ctx, &session.email).await
}

pub fn handle_oauth_login(provider: OAuthProvider) {
    let url = SupabaseClient::get_oauth_url(provider.as_str());

    if let Some(window) = web_sys::window() {
        let _ = window.location().set_href(&url);
    }
}
