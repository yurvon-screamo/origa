mod app;
mod demo_data;
mod pages;
mod repository;
mod routes;
mod ui_components;

use app::*;
use leptos::prelude::*;
use leptos_meta::MetaTags;
use leptos_router::components::Router;

fn main() {
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
