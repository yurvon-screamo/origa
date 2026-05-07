use std::collections::HashSet;

use crate::i18n::{t, use_i18n};
use crate::pages::lesson::LessonContext;
use crate::ui_components::{MarkdownText, MarkdownVariant};
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos::wasm_bindgen::JsCast;
use leptos_use::use_event_listener;
use origa::domain::{TokenTranslation, lookup_tokens_translations, tokenize_text};

const HINT_SEEN_KEY: &str = "origa_translator_hint_seen";
const HINT_TTL_MS: u64 = 30 * 24 * 60 * 60 * 1000;

fn has_kanji(text: &str) -> bool {
    text.chars().any(|c| {
        matches!(
            c,
            '\u{4E00}'..='\u{9FFF}' | '\u{3400}'..='\u{4DBF}' | '\u{F900}'..='\u{FAFF}'
        )
    })
}

fn get_hint_seen() -> bool {
    let Some(value) = web_sys::window()
        .and_then(|w| w.local_storage().ok())
        .flatten()
        .and_then(|ls| ls.get_item(HINT_SEEN_KEY).ok())
        .flatten()
    else {
        return false;
    };
    if value == "true" {
        return true;
    }
    let Ok(timestamp) = value.parse::<u64>() else {
        return false;
    };
    let now = js_sys::Date::now() as u64;
    now.saturating_sub(timestamp) < HINT_TTL_MS
}

fn set_hint_seen() {
    if let Some(ls) = web_sys::window()
        .and_then(|w| w.local_storage().ok())
        .flatten()
    {
        let now = js_sys::Date::now() as u64;
        let _ = ls.set_item(HINT_SEEN_KEY, &now.to_string());
    }
}

fn dismiss_hint(dismissed: RwSignal<bool>) {
    dismissed.set(true);
    set_hint_seen();
}

#[component]
pub fn TranslatorText(
    #[prop(into)] text: String,
    #[prop(optional, into, default = String::new().into())] class: Signal<String>,
    #[prop(optional, into)] test_id: Signal<String>,
) -> impl IntoView {
    let test_id_val = move || {
        let val = test_id.get();
        if val.is_empty() { None } else { Some(val) }
    };

    let Some(ctx) = use_context::<LessonContext>() else {
        return view! {
            <span class=move || format!("translator-text {}", class.get()) data-testid=test_id_val>
                <span class="translator-loading font-serif">{text.clone()}</span>
            </span>
        }
        .into_any();
    };

    let i18n = use_i18n();
    let translations: RwSignal<Vec<TokenTranslation>> = RwSignal::new(vec![]);
    let expanded: RwSignal<Option<usize>> = RwSignal::new(None);
    let is_loaded: RwSignal<bool> = RwSignal::new(false);
    let show_hint: RwSignal<bool> = RwSignal::new(!get_hint_seen());
    let container_ref = NodeRef::<leptos::html::Span>::new();

    let text_for_spawn = text.clone();
    let native_lang = ctx.native_language;
    spawn_local(async move {
        let lang = native_lang.get();
        let tokens = tokenize_text(&text_for_spawn).unwrap_or_default();
        translations.set(lookup_tokens_translations(&tokens, &lang));
        is_loaded.set(true);
    });

    if !get_hint_seen() {
        let dismissed = show_hint;
        spawn_local(async move {
            gloo_timers::future::TimeoutFuture::new(6000).await;
            if !dismissed.get() {
                dismiss_hint(dismissed);
            }
        });
    }

    let _ = use_event_listener(document(), leptos::ev::click, {
        move |ev: leptos::ev::MouseEvent| {
            let mut should_close = true;
            if let Some(el) = container_ref.get()
                && let Some(target) = ev.target()
            {
                let target_node: Option<web_sys::Node> = target.dyn_into().ok();
                let el_node: &web_sys::Node = &el;
                should_close = !el_node.contains(target_node.as_ref());
            }
            if should_close {
                expanded.set(None);
                if show_hint.get() {
                    dismiss_hint(show_hint);
                }
            }
        }
    });

    let _ = use_event_listener(
        document(),
        leptos::ev::keydown,
        move |ev: leptos::ev::KeyboardEvent| {
            if ev.key() == "Escape" {
                expanded.set(None);
            }
        },
    );

    let indexed = move || -> Vec<(usize, TokenTranslation)> {
        translations.get().into_iter().enumerate().collect()
    };

    let first_clickable_idx = move || {
        translations
            .get()
            .iter()
            .position(|t| t.pos.is_vocabulary_word())
    };

    view! {
        <span
            class=move || format!("translator-text {}", class.get())
            node_ref=container_ref
            data-testid=test_id_val
        >
            <Show
                when=move || is_loaded.get()
                fallback=move || view! {
                    <span class="translator-loading font-serif">{text.clone()}</span>
                }
            >
                <For
                    each=indexed
                    key=|(idx, _)| *idx
                    children=move |(idx, token): (usize, TokenTranslation)| {
                        let surface = token.surface_form.clone();
                        let reading = token.reading.clone();
                        let base_form = token.base_form.clone();
                        let translation_text = token.translation.clone();
                        let clickable = token.pos.is_vocabulary_word();
                        let has_kanji = has_kanji(&surface);
                        let show_base = base_form != surface;

                        let surface_view = if has_kanji {
                            view! {
                                <ruby class="furigana-ruby">
                                    {surface.clone()}
                                    <rp>"("</rp>
                                    <rt class="furigana-rt">{reading.clone()}</rt>
                                    <rp>")"</rp>
                                </ruby>
                            }.into_any()
                        } else {
                            view! { <span>{surface.clone()}</span> }.into_any()
                        };

                        if clickable {
                            view! {
                                <span class=move || {
                                    if expanded.get() == Some(idx) {
                                        "token-word token-word-active"
                                    } else {
                                        "token-word"
                                    }
                                }>
                                    <span
                                        class="token-surface"
                                        tabindex="0"
                                        on:click=move |ev: leptos::ev::MouseEvent| {
                                            ev.stop_propagation();
                                            expanded.update(|e| {
                                                *e = if *e == Some(idx) { None } else { Some(idx) };
                                            });
                                        }
                                        on:keydown=move |ev: leptos::ev::KeyboardEvent| {
                                            if ev.key() == "Enter" || ev.key() == " " {
                                                ev.prevent_default();
                                                expanded.update(|e| {
                                                    *e = if *e == Some(idx) { None } else { Some(idx) };
                                                });
                                            }
                                        }
                                    >
                                        {surface_view}
                                    </span>
                                    {move || {
                                        if expanded.get() == Some(idx) {
                                            view! {
                                                <div class="token-popup" on:click=move |ev: leptos::ev::MouseEvent| ev.stop_propagation()>
                                                    <div class="token-popup-surface">{surface.clone()}</div>
                                                    <div class="token-popup-reading">{reading.clone()}</div>
                                                    {if show_base {
                                                        view! {
                                                            <div class="token-popup-reading">{base_form.clone()}</div>
                                                        }.into_any()
                                                    } else {
                                                        ().into_any()
                                                    }}
                                                    <MarkdownText
                                                        content=Signal::derive({
                                                            let text = translation_text.clone();
                                                            move || text.clone().unwrap_or_default()
                                                        })
                                                        known_kanji=HashSet::new()
                                                        variant=Signal::derive(|| MarkdownVariant::Compact)
                                                        furigana=false
                                                    />
                                                </div>
                                            }.into_any()
                                        } else {
                                            ().into_any()
                                        }
                                    }}
                                </span>
                            }.into_any()
                        } else {
                            view! {
                                <span class="token-plain">{surface}</span>
                            }.into_any()
                        }
                    }
                />
                <Show when=move || show_hint.get() && is_loaded.get()>
                    {move || {
                        let hint_idx = first_clickable_idx();
                        hint_idx.map(|_| {
                            view! {
                                <span class="translator-hint" role="status" aria-live="polite">
                                    {t!(i18n, translator.hint_text)}
                                </span>
                            }.into_any()
                        }).unwrap_or_else(|| ().into_any())
                    }}
                </Show>
            </Show>
        </span>
    }.into_any()
}
