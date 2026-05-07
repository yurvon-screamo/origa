use crate::pages::lesson::LessonContext;
use crate::ui_components::{Tag, TagVariant};
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos::wasm_bindgen::JsCast;
use leptos_use::use_event_listener;
use origa::domain::{PartOfSpeech, TokenTranslation, lookup_tokens_translations, tokenize_text};

fn pos_display(pos: &PartOfSpeech) -> &'static str {
    match pos {
        PartOfSpeech::Verb => "verb",
        PartOfSpeech::Noun => "noun",
        PartOfSpeech::IAdjective => "i-adj",
        PartOfSpeech::NaAdjective => "na-adj",
        PartOfSpeech::Adverb => "adverb",
        PartOfSpeech::Particle => "particle",
        PartOfSpeech::AuxiliaryVerb => "aux",
        PartOfSpeech::Pronoun => "pronoun",
        PartOfSpeech::ProperNoun => "name",
        PartOfSpeech::Numeral => "num",
        PartOfSpeech::Conjunction => "conj",
        PartOfSpeech::PreNounAdjectival => "pre-adj",
        PartOfSpeech::Interjection => "interj",
        PartOfSpeech::Determiner => "det",
        _ => "",
    }
}

fn pos_tag_variant(pos: &PartOfSpeech) -> TagVariant {
    match pos {
        PartOfSpeech::Verb | PartOfSpeech::AuxiliaryVerb => TagVariant::Terracotta,
        PartOfSpeech::IAdjective | PartOfSpeech::NaAdjective => TagVariant::Sage,
        PartOfSpeech::Particle => TagVariant::Olive,
        _ => TagVariant::Default,
    }
}

fn is_clickable(pos: &PartOfSpeech) -> bool {
    !matches!(
        pos,
        PartOfSpeech::Symbol | PartOfSpeech::Whitespace | PartOfSpeech::AuxiliarySymbol
    )
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

    let translations: RwSignal<Vec<TokenTranslation>> = RwSignal::new(vec![]);
    let expanded: RwSignal<Option<usize>> = RwSignal::new(None);
    let is_loaded: RwSignal<bool> = RwSignal::new(false);
    let container_ref = NodeRef::<leptos::html::Span>::new();

    let text_for_spawn = text.clone();
    let native_lang = ctx.native_language;
    spawn_local(async move {
        let lang = native_lang.get();
        let tokens = tokenize_text(&text_for_spawn).unwrap_or_default();
        translations.set(lookup_tokens_translations(&tokens, &lang));
        is_loaded.set(true);
    });

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
                        let pos = token.pos.clone();
                        let pos_label = pos_display(&pos);
                        let tag_variant = pos_tag_variant(&pos);
                        let show_base = base_form != surface;
                        let clickable = is_clickable(&pos);

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
                                        on:click=move |ev: leptos::ev::MouseEvent| {
                                            ev.stop_propagation();
                                            expanded.update(|e| {
                                                *e = if *e == Some(idx) { None } else { Some(idx) };
                                            });
                                        }
                                    >
                                        {surface}
                                    </span>
                                    {move || {
                                        if expanded.get() == Some(idx) {
                                            view! {
                                                <div class="token-popup" on:click=move |ev: leptos::ev::MouseEvent| ev.stop_propagation()>
                                                    <span class="token-popup-reading">{reading.clone()}</span>
                                                    {if show_base {
                                                        view! {
                                                            <span class="token-popup-base">{base_form.clone()}</span>
                                                        }.into_any()
                                                    } else {
                                                        ().into_any()
                                                    }}
                                                    <span class="token-popup-translation">
                                                        {translation_text.clone().unwrap_or_else(|| "—".to_string())}
                                                    </span>
                                                    {if !pos_label.is_empty() {
                                                        view! {
                                                            <Tag variant=Signal::derive(move || tag_variant)>
                                                                {pos_label}
                                                            </Tag>
                                                        }.into_any()
                                                    } else {
                                                        ().into_any()
                                                    }}
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
            </Show>
        </span>
    }.into_any()
}
