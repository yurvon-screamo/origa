use crate::ui_components::button::{Button, ButtonVariant};
use crate::ui_components::divider::Divider;
use crate::ui_components::search::Search;
use leptos::prelude::*;

#[derive(Clone, Debug)]

pub struct FooterLink {
    pub label: String,
    pub href: String,
}

#[derive(Clone, Debug)]

pub struct FooterSection {
    pub title: String,
    pub links: Vec<FooterLink>,
}

#[component]
pub fn Footer(
    #[prop(optional, into)] _brand: Signal<String>,
    #[prop(optional, into)] _sections: Signal<Vec<FooterSection>>,
    #[prop(optional, into)] _description: Signal<String>,
    #[prop(optional, into)] _newsletter_placeholder: Signal<String>,
    #[prop(optional, into)] test_id: Signal<String>,
) -> impl IntoView {
    let test_id_val = move || {
        let val = test_id.get();
        if val.is_empty() {
            None
        } else {
            Some(val)
        }
    };

    view! {
        <footer data-testid=test_id_val class="footer py-16 mt-24">
            <div class="w-full px-6">
                <div class="grid md:grid-cols-4 gap-12 mb-12">
                    <div>
                        <h4 class="font-serif text-2xl mb-4">{move || _brand.get()}</h4>
                        <p class="font-mono text-[10px] tracking-widest text-[var(--fg-muted)] leading-relaxed">
                            {move || _description.get()}
                        </p>
                    </div>

                    <For
                        each=move || _sections.get()
                        key=|section| section.title.clone()
                        children=move |section| {
                            let links = section.links.clone();
                            view! {
                                <div>
                                    <p class="font-mono text-[9px] tracking-widest text-[var(--fg-muted)] uppercase mb-4">
                                        {section.title}
                                    </p>
                                    <ul class="space-y-2">
                                        <For
                                            each=move || links.clone()
                                            key=|link| link.label.clone()
                                            children=move |link| {
                                                view! {
                                                    <li>
                                                        <a
                                                            href=link.href
                                                            class="font-mono text-xs hover:text-[var(--accent-terracotta)] transition-colors"
                                                        >
                                                            {link.label}
                                                        </a>
                                                    </li>
                                                }
                                            }
                                        />
                                    </ul>
                                </div>
                            }
                        }
                    />

                    <div>
                        <p class="font-mono text-[9px] tracking-widest text-[var(--fg-muted)] uppercase mb-4">"Newsletter"</p>
                        <Search
                            placeholder=move || _newsletter_placeholder.get()
                            class="!pl-4"
                            value=RwSignal::new(String::new())
                        />
                        <Button class="btn-sm w-full mt-3" variant=ButtonVariant::Filled>
                            "Subscribe"
                        </Button>
                    </div>
                </div>

                <Divider class="mb-8" />

                <div class="flex flex-col md:flex-row justify-between items-center gap-4">
                    <p class="font-mono text-[9px] tracking-widest text-[var(--fg-muted)]">
                        "MMXXIV LE STYLE DESIGN SYSTEM"
                    </p>
                    <div class="flex gap-6">
                        <a href="#" class="font-mono text-[9px] tracking-widest text-[var(--fg-muted)] hover:text-[var(--fg-black)] transition-colors">
                            "Privacy"
                        </a>
                        <a href="#" class="font-mono text-[9px] tracking-widest text-[var(--fg-muted)] hover:text-[var(--fg-black)] transition-colors">
                            "Terms"
                        </a>
                        <a href="#" class="font-mono text-[9px] tracking-widest text-[var(--fg-muted)] hover:text-[var(--fg-black)] transition-colors">
                            "Cookies"
                        </a>
                    </div>
                </div>
            </div>
        </footer>
    }
}
