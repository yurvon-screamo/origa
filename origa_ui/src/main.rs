use leptos::prelude::*;
use leptos_meta::MetaTags;
use leptos_router::components::Router;
use origa_ui::app::*;
use origa_ui::i18n::I18nContextProvider;

fn main() {
    origa_ui::init_tracing();

    mount_to_body(|| {
        view! {
            <MetaTags />
            <I18nContextProvider cookie_options=origa_ui::i18n::persistent_locale_cookie_options()>
                <Router>
                    <App/>
                </Router>
            </I18nContextProvider>
        }
    })
}
