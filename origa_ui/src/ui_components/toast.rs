use leptos::prelude::*;
use leptos::task::spawn_local;

#[derive(Clone, Copy, PartialEq, Default, Debug)]
pub enum ToastType {
    #[default]
    Info,
    Success,

    Warning,
    Error,
}

#[derive(Clone)]
pub struct ToastData {
    pub id: usize,
    pub toast_type: ToastType,
    pub title: String,
    pub message: String,
    pub duration_ms: Option<u64>,
}

#[component]
pub fn Toast(
    toast: ToastData,
    on_close: Callback<usize>,
    #[prop(optional)] duration_ms: u64,
) -> impl IntoView {
    let toast_id = toast.id;
    let actual_duration = toast.duration_ms.unwrap_or(duration_ms);
    let has_duration = actual_duration > 0;

    if has_duration {
        spawn_local(async move {
            gloo_timers::future::TimeoutFuture::new(actual_duration as u32).await;
            on_close.run(toast_id);
        });
    }

    view! {
        <div class=move || {
            let toast_class = match toast.toast_type {
                ToastType::Info => "toast-info",
                ToastType::Success => "toast-success",
                ToastType::Warning => "toast-warning",
                ToastType::Error => "toast-error",
            };
            format!("toast {}", toast_class)
        }>
            <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
                {move || match toast.toast_type {
                    ToastType::Success => view! {
                        <path d="M20 6L9 17l-5-5" />
                    }.into_any(),
                    ToastType::Warning => view! {
                        <path d="M12 9v4m0 4h.01M12 3l9 18H3L12 3z" />
                    }.into_any(),
                    ToastType::Error => view! {
                        <><circle cx="12" cy="12" r="10" />
                        <path d="M15 9l-6 6m0-6l6 6" /></>
                    }.into_any(),
                    ToastType::Info => view! {
                        <><circle cx="12" cy="12" r="10" />
                        <path d="M12 16v-4m0-4h.01" /></>
                    }.into_any(),
                }}
            </svg>
            <div class="flex-1">
                <p class="font-mono text-xs tracking-wider">{toast.title}</p>
                <p class="font-mono text-[10px] text-[var(--fg-muted)] mt-1">{toast.message}</p>
            </div>
            <button
                class="toast-close"
                on:click=move |_| on_close.run(toast_id)
            >
                <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                    <path d="M18 6L6 18M6 6l12 12" />
                </svg>
            </button>
        </div>
    }
}

#[component]
pub fn ToastContainer(
    toasts: RwSignal<Vec<ToastData>>,
    #[prop(optional)] duration_ms: u64,
) -> impl IntoView {
    let on_close = Callback::new(move |id: usize| {
        toasts.update(|t| t.retain(|toast| toast.id != id));
    });

    view! {
        <div class="toast-container">
            <For
                each=move || toasts.get()
                key=|toast| toast.id
                children=move |toast| {
                    view! {
                        <Toast toast=toast on_close=on_close duration_ms=duration_ms />
                    }
                }
            />
        </div>
    }
}
