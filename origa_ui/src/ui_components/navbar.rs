use crate::ui_components::button::{Button, ButtonSize, ButtonVariant};
use leptos::either::*;
use leptos::prelude::*;

#[derive(Clone, Debug)]

pub struct NavItem {
    pub label: String,
    pub href: String,
}

#[component]
pub fn Navbar(
    #[prop(optional, into)] _brand: Signal<String>,
    #[prop(optional, into)] _items: Signal<Vec<NavItem>>,
    #[prop(into)] _cart_count: RwSignal<u32>,
    #[prop(optional)] _on_sign_in: Option<Callback<leptos::ev::MouseEvent>>,
    #[prop(optional, into)] test_id: Signal<String>,
) -> impl IntoView {
    let test_id_val = move || {
        let val = test_id.get();
        if val.is_empty() { None } else { Some(val) }
    };

    let signin_test_id = Signal::derive(move || {
        let val = test_id.get();
        if val.is_empty() {
            String::new()
        } else {
            format!("{}-signin", val)
        }
    });

    let cart_test_id = Signal::derive(move || {
        let val = test_id.get();
        if val.is_empty() {
            String::new()
        } else {
            format!("{}-cart", val)
        }
    });
    view! {
        <nav class="navbar sticky top-0 z-50 pt-safe-t" data-testid=test_id_val>
            <div class="w-full px-6 flex items-center justify-between">
                <a href="#" class="font-serif text-2xl">{move || _brand.get()}</a>

                <div class="hidden md:flex items-center gap-8">
                    <For
                        each=move || _items.get()
                        key=|item| item.label.clone()
                        children=move |item| {
                            view! {
                                <a
                                    href=item.href
                                    class="font-mono text-xs tracking-widest uppercase hover:text-[var(--accent-terracotta)] transition-colors"
                                >
                                    {item.label}
                                </a>
                            }
                        }
                    />
                </div>

                <div class="flex items-center gap-4">
                    {if let Some(click_handler) = _on_sign_in {
                        Either::Left(view! {
                            <Button variant=ButtonVariant::Ghost size=ButtonSize::Small on_click=click_handler test_id=signin_test_id>
                                "Sign In"
                            </Button>
                        })
                    } else {
                        Either::Right(view! {
                            <Button variant=ButtonVariant::Ghost size=ButtonSize::Small test_id=signin_test_id>
                                "Sign In"
                            </Button>
                        })
                    }}
                    <Button variant=ButtonVariant::Filled size=ButtonSize::Small test_id=cart_test_id>
                        "Cart (" {move || _cart_count.get()} ")"
                    </Button>
                </div>
            </div>
        </nav>
    }
}
