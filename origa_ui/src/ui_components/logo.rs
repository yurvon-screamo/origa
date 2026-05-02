use leptos::prelude::*;

use crate::core::config::public_url;

#[derive(Clone, Copy, PartialEq, Default, Debug)]
pub enum LogoSize {
    #[default]
    Sm,
    Lg,
}

impl LogoSize {
    fn src(&self) -> String {
        match self {
            LogoSize::Sm => public_url("/public/logo-32.png"),
            LogoSize::Lg => public_url("/public/logo-128.png"),
        }
    }

    fn dimensions(&self) -> (u32, u32) {
        match self {
            LogoSize::Sm => (32, 32),
            LogoSize::Lg => (96, 96),
        }
    }
}

#[component]
pub fn Logo(
    size: LogoSize,
    #[prop(optional, into)] test_id: Signal<String>,
    #[prop(optional, into)] class: Signal<String>,
) -> impl IntoView {
    let test_id_val = move || {
        let val = test_id.get();
        if val.is_empty() { None } else { Some(val) }
    };

    view! {
        <img
            src=size.src()
            width=size.dimensions().0
            height=size.dimensions().1
            alt="Origa"
            class=move || class.get()
            data-testid=test_id_val
        />
    }
}
