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
    #[prop(optional, into)] alert_type: Signal<AlertType>,
    #[prop(optional, into)] title: Signal<String>,
    #[prop(optional, into)] message: Signal<String>,
    #[prop(optional, into)] class: Signal<String>,
) -> impl IntoView {
    view! {
        <div class=move || {
            let alert_class = match alert_type.get() {
                AlertType::Info => "alert-info",
                AlertType::Success => "alert-success",
                AlertType::Warning => "alert-warning",
                AlertType::Error => "alert-error",
            };
            format!("alert {} {}", alert_class, class.get())
        }>
            <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
                {move || match alert_type.get() {
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
                <p class="font-mono text-xs tracking-wider">{move || title.get()}</p>
                <p class="font-mono text-[10px] text-[var(--fg-muted)] mt-1">{move || message.get()}</p>
            </div>
        </div>
    }
}
