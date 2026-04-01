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
        <footer data-testid=test_id_val class="footer">
            <div class="footer-container w-full px-6">
                <div class="footer-grid">
                    <div class="footer-section">
                        <h4 class="footer-brand">{move || _brand.get()}</h4>
                        <p class="footer-tagline">
                            {move || _description.get()}
                        </p>
                    </div>

                    <For
                        each=move || _sections.get()
                        key=|section| section.title.clone()
                        children=move |section| {
                            let links = section.links.clone();
                            view! {
                                <div class="footer-section">
                                    <p class="footer-section-title">
                                        {section.title}
                                    </p>
                                    <ul class="footer-nav">
                                        <For
                                            each=move || links.clone()
                                            key=|link| link.label.clone()
                                            children=move |link| {
                                                view! {
                                                    <li>
                                                        <a
                                                            href=link.href
                                                            class="footer-link"
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

                    <div class="footer-section">
                        <p class="footer-section-title">"Newsletter"</p>
                        <Search
                            placeholder=move || _newsletter_placeholder.get()
                            class="!pl-4"
                            value=RwSignal::new(String::new())
                        />
                        <Button class="btn btn-sm w-full mt-3" variant=ButtonVariant::Filled>
                            "Subscribe"
                        </Button>
                    </div>
                </div>

                <Divider class="mb-8" />

                <div class="footer-bottom">
                    <p class="footer-copyright">
                        "MMXXIV LE STYLE DESIGN SYSTEM"
                    </p>
                    <div class="footer-social">
                        <a href="#" class="footer-social-link">
                            "Privacy"
                        </a>
                        <a href="#" class="footer-social-link">
                            "Terms"
                        </a>
                        <a href="#" class="footer-social-link">
                            "Cookies"
                        </a>
                    </div>
                </div>
            </div>
        </footer>
    }
}
