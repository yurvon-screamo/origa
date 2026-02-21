use leptos::prelude::*;
use leptos_use::use_event_listener;

#[component]
pub fn Modal(
    #[prop(optional)] is_open: RwSignal<bool>,
    #[prop(optional)] on_close: Option<Callback<leptos::ev::MouseEvent>>,
    #[prop(optional, into)] title: Signal<String>,
    children: ChildrenFn,
) -> impl IntoView {
    let close_modal = move |ev: leptos::ev::MouseEvent| {
        is_open.set(false);
        if let Some(on_close) = on_close {
            on_close.run(ev);
        }
    };

    let _ = use_event_listener(
        document(),
        leptos::ev::keydown,
        move |ev: leptos::ev::KeyboardEvent| {
            if ev.key() == "Escape" {
                is_open.set(false);
            }
        },
    );

    view! {
        <Show when=move || is_open.get()>
            <>
                <div
                    class="modal-backdrop"
                    on:click=close_modal
                ></div>
                <div class="modal-content">
                    <div class="flex justify-between items-start mb-6">
                        <div>
                            <span class="font-mono text-[9px] tracking-widest text-[var(--fg-muted)]">"MODAL DIALOG"</span>
                            <h3 class="font-serif text-2xl mt-1">{move || title.get()}</h3>
                        </div>
                        <button
                            on:click=close_modal
                            class="text-[var(--fg-muted)] hover:text-[var(--fg-black)] transition-colors"
                        >
                            <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
                                <path d="M18 6L6 18M6 6l12 12" />
                            </svg>
                        </button>
                    </div>
                    {children()}
                </div>
            </>
        </Show>
    }
}
