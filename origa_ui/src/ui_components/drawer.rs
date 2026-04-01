use leptos::prelude::*;
use leptos_use::use_event_listener;

#[component]
pub fn Drawer(
    #[prop(optional)] is_open: RwSignal<bool>,
    #[prop(optional)] on_close: Option<Callback<leptos::ev::MouseEvent>>,
    #[prop(optional, into)] title: Signal<String>,
    #[prop(optional)] action_button: Option<ChildrenFn>,
    #[prop(optional, into)] test_id: Signal<String>,
    children: ChildrenFn,
) -> impl IntoView {
    let close_drawer = move |ev: leptos::ev::MouseEvent| {
        is_open.set(false);
        if let Some(on_close) = on_close {
            on_close.run(ev);
        }
    };

    let test_id_val = move || {
        let val = test_id.get();
        if val.is_empty() {
            None
        } else {
            Some(val)
        }
    };

    let test_id_close = move || {
        let val = test_id.get();
        if val.is_empty() {
            None
        } else {
            Some(format!("{}-close", val))
        }
    };

    let test_id_backdrop = move || {
        let val = test_id.get();
        if val.is_empty() {
            None
        } else {
            Some(format!("{}-backdrop", val))
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

    let children = StoredValue::new(children);
    let action_button = StoredValue::new(action_button);

    view! {
        <Show when=move || is_open.get()>
            <>
                {/* Используем тот же бэкдроп, что и в модалке, для консистентности */}
                <div
                    class="modal-backdrop anima-backdrop-enter"
                    on:click=close_drawer
                    data-testid=test_id_backdrop
                ></div>

                <div class="drawer-content" data-testid=test_id_val>
                    {/* Header: фиксированный */}
                    <div class="modal-header">
                        <div>
                            <h3 class="font-serif text-2xl mt-1">{move || title.get()}</h3>
                        </div>
                        <div class="flex items-center gap-2">
                            {move || action_button.with_value(|b| b.as_ref().map(|c| c()))}
                            <button
                                on:click=close_drawer
                                class="modal-close-btn"
                                data-testid=test_id_close
                            >
                                <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
                                    <path d="M18 6L6 18M6 6l12 12" />
                                </svg>
                            </button>
                        </div>
                    </div>

                    {/* Body: скроллируемый контент */}
                    <div class="drawer-body">
                        {move || children.with_value(|c| c())}
                    </div>
                </div>
            </>
        </Show>
    }
}
