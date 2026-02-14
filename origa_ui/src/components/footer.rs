use crate::components::button::{Button, ButtonVariant};
use crate::components::divider::Divider;
use crate::components::search::Search;
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
    #[prop(into)] brand: String,
    #[prop(into)] sections: Vec<FooterSection>,
    #[prop(into)] description: String,
    #[prop(optional, into)] newsletter_placeholder: String,
) -> impl IntoView {
    view! {
        <footer class="footer py-16 mt-24">
            <div class="max-w-6xl mx-auto px-6">
                <div class="grid md:grid-cols-4 gap-12 mb-12">
                    <div>
                        <h4 class="font-serif text-2xl mb-4">{brand}</h4>
                        <p class="font-mono text-[10px] tracking-widest text-[var(--fg-muted)] leading-relaxed">
                            {description}
                        </p>
                    </div>

                    <For
                        each=move || sections.clone()
                        key=|section| section.title.clone()
                        children=move |section| {
                            view! {
                                <div>
                                    <p class="font-mono text-[9px] tracking-widest text-[var(--fg-muted)] uppercase mb-4">
                                        {section.title}
                                    </p>
                                    <ul class="space-y-2">
                                        <For
                                            each=move || section.links.clone()
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
                            placeholder=newsletter_placeholder
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
