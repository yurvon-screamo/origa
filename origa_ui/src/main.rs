use leptos::prelude::*;
use leptos_meta::MetaTags;
use leptos_router::components::Router;
use origa_ui::app::*;

fn main() {
    origa_ui::init_tracing();
    console_error_panic_hook::set_once();

    mount_to_body(|| {
        view! {
            <MetaTags />
            <Router>
                <App/>
            </Router>
        }
    })
}
