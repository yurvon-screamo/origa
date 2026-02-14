use leptos::prelude::*;

#[derive(Clone, Copy, PartialEq, Default, Debug)]
pub enum AlertType {
    #[default]
    Info,
    Success,
    Warning,
    Error,
}

#[component]
pub fn Alert(
    #[prop(optional)] alert_type: AlertType,
    #[prop(optional, into)] title: String,
    #[prop(optional, into)] message: String,
    #[prop(optional, into)] class: String,
) -> impl IntoView {
    let alert_class = match alert_type {
        AlertType::Info => "alert-info",
        AlertType::Success => "alert-success",
        AlertType::Warning => "alert-warning",
        AlertType::Error => "alert-error",
    };

    let full_class = format!("alert {} {}", alert_class, class);

    view! {
        <div class=full_class>
            <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
                {match alert_type {
                    AlertType::Success => view! {
                        <path d="M20 6L9 17l-5-5" />
                    }.into_any(),
                    AlertType::Warning => view! {
                        <path d="M12 9v4m0 4h.01M12 3l9 18H3L12 3z" />
                    }.into_any(),
                    AlertType::Error => view! {
                        <><circle cx="12" cy="12" r="10" />
                        <path d="M15 9l-6 6m0-6l6 6" /></>
                    }.into_any(),
                    AlertType::Info => view! {
                        <><circle cx="12" cy="12" r="10" />
                        <path d="M12 16v-4m0-4h.01" /></>
                    }.into_any(),
                }}
            </svg>
            <div>
                <p class="font-mono text-xs tracking-wider">{title}</p>
                <p class="font-mono text-[10px] text-[var(--fg-muted)] mt-1">{message}</p>
            </div>
        </div>
    }
}
