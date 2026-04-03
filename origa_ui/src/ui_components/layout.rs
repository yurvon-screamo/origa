use leptos::prelude::*;

#[derive(Clone, Copy, PartialEq, Debug, Default)]
pub enum PageLayoutVariant {
    #[default]
    Centered,
    Full,

    Compact,
}

#[component]
pub fn PageLayout(
    #[prop(optional, into)] variant: Signal<PageLayoutVariant>,
    #[prop(optional, into, default = "w-full px-4 sm:px-6 lg:px-8".to_string().into())]
    container_class: Signal<String>,
    #[prop(optional, into)] test_id: Signal<String>,
    children: Children,
) -> impl IntoView {
    let test_id_val = move || {
        let val = test_id.get();
        if val.is_empty() { None } else { Some(val) }
    };

    view! {
        <div
            data-testid=test_id_val
            class=move || {
                match variant.get() {
                    PageLayoutVariant::Centered => "page-layout-centered",
                    PageLayoutVariant::Full => "page-layout-full",
                    PageLayoutVariant::Compact => "page-layout-compact",
                }
            }
        >
            <div class=move || container_class.get()>
                {children()}
            </div>
        </div>
    }
}

#[derive(Clone, Copy, PartialEq, Debug, Default)]
pub enum CardLayoutSize {
    Small,

    Medium,

    Large,
    #[default]
    Adaptive,
}

#[component]
pub fn CardLayout(
    #[prop(optional, into)] size: Signal<CardLayoutSize>,
    #[prop(optional, into)] class: Signal<String>,
    #[prop(optional, into)] test_id: Signal<String>,
    children: Children,
) -> impl IntoView {
    let test_id_val = move || {
        let val = test_id.get();
        if val.is_empty() { None } else { Some(val) }
    };

    view! {
        <div
            data-testid=test_id_val
            class=move || {
                let base_class = match size.get() {
                    CardLayoutSize::Small => "card-layout-small",
                    CardLayoutSize::Medium => "card-layout-medium",
                    CardLayoutSize::Large => "card-layout-large",
                    CardLayoutSize::Adaptive => "card-layout-adaptive",
                };
                format!("{} {}", base_class, class.get())
            }
        >
            <div class="card-layout-content">
                {children()}
            </div>
        </div>
    }
}
