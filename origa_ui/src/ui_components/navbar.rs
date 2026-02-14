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
    #[prop(into)] brand: String,
    #[prop(into)] items: Vec<NavItem>,
    #[prop(into)] cart_count: RwSignal<u32>,
    #[prop(optional)] on_sign_in: Option<Callback<leptos::ev::MouseEvent>>,
) -> impl IntoView {
    view! {
        <nav class="navbar sticky top-0 z-50">
            <div class="max-w-6xl mx-auto px-6 flex items-center justify-between">
                <a href="#" class="font-serif text-2xl">{brand}</a>

                <div class="hidden md:flex items-center gap-8">
                    <For
                        each=move || items.clone()
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
                    {if let Some(click_handler) = on_sign_in {
                        Either::Left(view! {
                            <Button variant=ButtonVariant::Ghost size=ButtonSize::Small on_click=click_handler>
                                "Sign In"
                            </Button>
                        })
                    } else {
                        Either::Right(view! {
                            <Button variant=ButtonVariant::Ghost size=ButtonSize::Small>
                                "Sign In"
                            </Button>
                        })
                    }}
                    <Button variant=ButtonVariant::Filled size=ButtonSize::Small>
                        "Cart (" {move || cart_count.get()} ")"
                    </Button>
                </div>
            </div>
        </nav>
    }
}
