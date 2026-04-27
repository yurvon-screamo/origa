use leptos::prelude::*;

#[derive(Clone, Copy, PartialEq, Default, Debug)]
pub enum LogoSize {
    #[default]
    Sm,
    Lg,
}

impl LogoSize {
    fn src(&self) -> &'static str {
        match self {
            LogoSize::Sm => "/logo-32.png",
            LogoSize::Lg => "/logo-128.png",
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
