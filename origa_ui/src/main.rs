use app::*;
use leptos::prelude::*;
use thaw::*;

mod app;
mod components;
mod hooks;
mod services;
mod views;

fn main() {
    console_error_panic_hook::set_once();
    mount_to_body(|| {
        view! {
            <ConfigProvider>
                <App/>
            </ConfigProvider>
        }
    })
}
