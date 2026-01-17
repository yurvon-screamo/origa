use dioxus::prelude::*;
use dioxus_logger::tracing::Level;
use ulid::Ulid;

use origa::{
    application::UserRepository,
    domain::{JapaneseLevel, NativeLanguage, User},
    settings::ApplicationEnvironment,
};

use views::{
    Grammar, GrammarReference, Import, Kanji, KanjiCards, Learn, Navbar, Overview, Profile,
    Vocabulary,
};

pub const DEFAULT_USERNAME: &str = "yurvon_screamo";

pub async fn ensure_user(
    env: &'static ApplicationEnvironment,
    username: &str,
) -> Result<Ulid, String> {
    let repo = env.get_repository().await.map_err(to_error)?;
    if let Some(user) = repo
        .list()
        .await
        .map_err(to_error)?
        .into_iter()
        .find(|x| x.username() == username)
    {
        return Ok(user.id());
    }
    let new_user = User::new(
        username.to_string(),
        JapaneseLevel::N5,
        NativeLanguage::Russian,
    );
    let id = new_user.id();
    repo.save(&new_user).await.map_err(to_error)?;
    Ok(id)
}

pub fn to_error(err: impl std::fmt::Display) -> String {
    err.to_string()
}

mod components;
mod domain;
mod views;

const APP_ICON: Asset = asset!("/assets/icons/32x32.png");
const MAIN_CSS: Asset = asset!("/assets/styling/main.css");
const TAILWIND_CSS: Asset = asset!("/assets/tailwind.css");
const UI_TOKENS_CSS: Asset = asset!("/assets/styles/tokens.css");
const UI_BASE_CSS: Asset = asset!("/assets/styles/base.css");
const UI_UTILITIES_CSS: Asset = asset!("/assets/styles/utilities.css");

fn main() {
    dioxus_logger::init(Level::INFO).expect("failed to init logger");
    dioxus::launch(App);
}

#[derive(Debug, Clone, Routable, PartialEq)]
#[rustfmt::skip]
enum Route {
    #[layout(Navbar)]
        #[route("/")]
        Learn {},
        #[route("/import")]
        Import {},
        #[route("/vocabulary")]
        Vocabulary {},
        #[route("/kanji")]
        Kanji {},
        #[route("/kanji-cards")]
        KanjiCards {},
        #[route("/grammar-reference")]
        GrammarReference {},
        #[route("/grammar")]
        Grammar {},
        #[route("/overview")]
        Overview {},
        #[route("/profile")]
        Profile {},
}

#[component]
fn App() -> Element {
    rsx! {
        document::Link { rel: "icon", href: APP_ICON }
        document::Link { rel: "stylesheet", href: MAIN_CSS }
        document::Link { rel: "stylesheet", href: TAILWIND_CSS }
        document::Link { rel: "stylesheet", href: UI_TOKENS_CSS }
        document::Link { rel: "stylesheet", href: UI_BASE_CSS }
        document::Link { rel: "stylesheet", href: UI_UTILITIES_CSS }
        style { {global_styles()} }
        Router::<Route> {}
    }
}

fn global_styles() -> &'static str {
    r#"
                /* moved to assets/styles/base.css */
    "#
}
