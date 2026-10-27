use std::collections::HashSet;

use crate::pages::lesson::LessonContext;
use crate::ui_components::{MarkdownText, MarkdownVariant};
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos::wasm_bindgen::JsCast;
use leptos_use::use_event_listener;
use origa::domain::{
    NativeLanguage, TokenTranslation, lookup_precomputed, lookup_tokens_translations, tokenize_text,
};

use crate::feedback::{FeedbackCategory, FeedbackContext, FeedbackSource, FeedbackSubject};
use crate::i18n::{t, use_i18n};
use crate::loaders::dictionary::ensure_tokenizer_loaded;

/// The word a quick-add click creates a card for: the dictionary form
/// (base_form) — the tokenizer guarantees a non-empty base for vocabulary
/// tokens (surface when normalized is `*`/empty); the surface fallback is
/// a defensive guard, not an expected path.
fn add_word_target(base_form: &str, surface: &str) -> String {
    let base = base_form.trim();
    if base.is_empty() {
        surface.to_string()
    } else {
        base.to_string()
    }
}

fn has_kanji(text: &str) -> bool {
    text.chars().any(|c| {
        matches!(
            c,
            '\u{4E00}'..='\u{9FFF}' | '\u{3400}'..='\u{4DBF}' | '\u{F900}'..='\u{FAFF}'
        )
    })
}

/// Render phase of the translator: while the tokenizer dictionary warms
/// up (#521) the component shows an explicit loader instead of silently
/// rendering bare text — the wait is a one-off operation and must not
/// look like the final state.
#[derive(Clone, Copy, PartialEq, Debug)]
enum TranslatorPhase {
    WaitingDictionary,
    Ready,
    PlainText,
}

#[component]
pub fn TranslatorText(
    #[prop(into)] text: String,
    #[prop(optional, into, default = String::new().into())] class: Signal<String>,
    #[prop(optional, into)] test_id: Signal<String>,
    #[prop(optional, into)] native_language: Option<Signal<NativeLanguage>>,
) -> impl IntoView {
    let test_id_val = move || {
        let val = test_id.get();
        if val.is_empty() { None } else { Some(val) }
    };

    let native_lang: Option<Signal<NativeLanguage>> = native_language
        .or_else(|| use_context::<LessonContext>().map(|ctx| ctx.native_language.into()));

    let Some(native_lang) = native_lang else {
        return view! {
            <span
                class=move || format!("translator-text {}", class.get())
                data-testid=test_id_val
            >
                <span class="translator-loading font-serif">{text.clone()}</span>
            </span>
        }
        .into_any();
    };

    let translations: RwSignal<Vec<TokenTranslation>> = RwSignal::new(vec![]);
    let expanded: RwSignal<Option<usize>> = RwSignal::new(None);
    // StoredValue (Copy) so the per-token `children` closure can capture it
    // without moving the Option out of the root children closure (Fn bound).
    let feedback_context = StoredValue::new(use_context::<FeedbackContext>());
    // Quick-add (#528+): repository comes from the app shell like
    // FeedbackContext — isolated component mounts see no add button.
    // StoredValue (Copy) keeps the per-token `children` closure Fn-safe.
    let repository = StoredValue::new(use_context::<crate::repository::HybridUserRepository>());
    // StoredValue (Copy) so the per-token `children` closure can capture it
    // without moving the String out of the root children closure (Fn bound).
    let source_line = StoredValue::new(text.clone());
    let phase: RwSignal<TranslatorPhase> = RwSignal::new(TranslatorPhase::WaitingDictionary);
    let container_ref = NodeRef::<leptos::html::Span>::new();
    let i18n = use_i18n();
    // i18n context at component scope — the report entry reads the LIVE
    // locale when it fires (a mid-session language switch is reflected);
    // StoredValue keeps the per-token children closure Fn-compatible.
    let report_i18n = StoredValue::new(i18n);

    let text_for_spawn = text.clone();
    spawn_local(async move {
        let lang = native_lang.get();
        // Precompute path (#521): phrases and card words carry offline
        // tokens — no tokenizer dictionary involved.
        if let Some(entry) = lookup_precomputed(&text_for_spawn)
            && !entry.tokens.is_empty()
        {
            let tokens: Vec<_> = entry.tokens.iter().map(|t| t.to_token_info()).collect();
            translations.set(lookup_tokens_translations(&tokens, &lang, &text_for_spawn));
            phase.set(TranslatorPhase::Ready);
            return;
        }

        // Live path: gate on tokenizer readiness (it left the startup
        // overlay and may still be warming up). A failed load or empty
        // tokenization settles on the plain-text render — the original
        // string stays visible instead of a blank span.
        match ensure_tokenizer_loaded()
            .await
            .and_then(|()| tokenize_text(&text_for_spawn))
        {
            Ok(tokens) if !tokens.is_empty() => {
                translations.set(lookup_tokens_translations(&tokens, &lang, &text_for_spawn));
                phase.set(TranslatorPhase::Ready);
            },
            _ => phase.set(TranslatorPhase::PlainText),
        }
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
                when=move || phase.get() == TranslatorPhase::Ready
                fallback=move || {
                    if phase.get() == TranslatorPhase::WaitingDictionary {
                        view! {
                            // One-off dictionary warmup (#521): an explicit
                            // loader, never bare text — silence would read
                            // as the final state.
                            <span
                                class="translator-loading translator-loading-dict"
                                data-testid="translator-dict-loading"
                            >
                                <span class="spinner spinner-sm" aria-hidden="true"></span>
                                {t!(i18n, ui.loading_dictionaries)}
                            </span>
                        }
                        .into_any()
                    } else {
                        // Final plain-text outcome (no tokens): the
                        // original string stays visible.
                        view! {
                            <span class="translator-loading font-serif">{text.clone()}</span>
                        }
                        .into_any()
                    }
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
                        let grammar_label = token.grammar_label.clone();
                        let grammar_description = token.grammar_description.clone();
                        let clickable = token.pos.is_vocabulary_word() || grammar_label.is_some();
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

                        // Store fields for the popup closure
                        let popup_data = StoredValue::new((
                            surface.clone(),
                            reading.clone(),
                            base_form.clone(),
                            grammar_label.clone(),
                            grammar_description.clone(),
                            translation_text.clone(),
                            show_base,
                        ));

                        // "Wrong translation?" entry point (ADR-055): opens
                        // the feedback modal with this exact token + the
                        // source line as the captured subject. Rendered only
                        // when the FeedbackContext is mounted (app shell);
                        // isolated component tests see no button. The subject
                        // lives in a StoredValue so the per-token children
                        // closure stays `Fn` (same pattern as popup_data).
                        let report_data = StoredValue::new((
                            surface.clone(),
                            reading.clone(),
                            source_line.get_value(),
                        ));
                        let report_i18n_data = report_i18n;
                        let on_report = feedback_context.get_value().as_ref().map(|feedback| {
                            let feedback = feedback.clone();
                            Callback::new(move |()| {
                                let (surface, reading, line) =
                                    report_data.with_value(|(s, r, l)| {
                                        (s.clone(), r.clone(), l.clone())
                                    });
                                let locale = report_i18n_data.get_value().get_locale().to_string();
                                // Close the popup first: the modal renders
                                // above it and the popup should not linger.
                                expanded.set(None);
                                feedback.open(
                                    FeedbackCategory::Translation,
                                    FeedbackSource::TranslatorPopup,
                                    FeedbackSubject {
                                        surface: surface.clone(),
                                        reading: (!reading.is_empty()).then(|| reading.clone()),
                                        context_line: (!line.is_empty()).then(|| line.clone()),
                                    },
                                    &locale,
                                );
                            })
                        });

                        // Quick-add (#528+): vocabulary tokens only — a
                        // particle/grammar token is not a deck candidate.
                        // Zero-state button (owner decision): no loading/
                        // added/duplicate states — the use case dedups,
                        // a double click is a harmless skip.
                        let on_add_word = repository.with_value(|repo| {
                            (repo.is_some() && token.pos.is_vocabulary_word())
                                .then(|| {
                                    let repository = repo.clone().unwrap();
                                    let target = add_word_target(&base_form, &surface);
                                    Callback::new(move |()| {
                                        let repository = repository.clone();
                                        let word = target.clone();
                                        // Close first (the report-button
                                        // pattern): the action is fired,
                                        // the sentence stays readable.
                                        expanded.set(None);
                                        spawn_local(async move {
                                            // The precomputed render path
                                            // shows tokens BEFORE the
                                            // background tokenizer load
                                            // finishes; from_text silently
                                            // swallows TokenizerError, so
                                            // an un-gated click would lose
                                            // the word with zero signals.
                                            // Gate first (the add_word_modal
                                            // pattern).
                                            if let Err(e) =
                                                ensure_tokenizer_loaded().await
                                            {
                                                tracing::warn!(
                                                    error = %e,
                                                    word = %word,
                                                    "tokenizer gate failed, word not added"
                                                );
                                                return;
                                            }
                                            let use_case = origa::use_cases::CreateVocabularyCardUseCase::new(
                                                &repository,
                                            );
                                            match use_case.execute(word.clone()).await {
                                                Ok(result) => {
                                                    // Zero-state UI (owner
                                                    // decision); skipped
                                                    // results are still
                                                    // worth a trace — the
                                                    // interjection/masu-stem
                                                    // class resolves
                                                    // differently in
                                                    // from_text than in the
                                                    // popup lookup.
                                                    if !result.skipped_no_translation.is_empty() {
                                                        tracing::warn!(
                                                            word = %word,
                                                            "word skipped: no dictionary translation"
                                                        );
                                                    }
                                                    if !result.skipped_duplicates.is_empty() {
                                                        tracing::debug!(
                                                            word = %word,
                                                            "word skipped: already in deck"
                                                        );
                                                    }
                                                },
                                                Err(e) => tracing::warn!(
                                                    error = %e,
                                                    word = %word,
                                                    "quick-add card creation failed"
                                                ),
                                            }
                                        });
                                    })
                                })
                        });

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
                                            let (s, r, bf, gl, gd, tt, sb) = popup_data.get_value();
                                            view! {
                                                <TokenPopup
                                                    surface=s
                                                    reading=r
                                                    base_form=Signal::derive(move || if sb { Some(bf.clone()) } else { None })
                                                    grammar_label=Signal::derive(move || gl.clone())
                                                    grammar_description=Signal::derive(move || gd.clone())
                                                    translation_text=Signal::derive(move || tt.clone())
                                                    on_add_word=on_add_word
                                                    on_report=on_report
                                                />
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

/// Popup that shows translation/grammar info for a token. Measures its
/// viewport position on mount and shifts horizontally to stay on-screen.
#[component]
fn TokenPopup(
    surface: String,
    reading: String,
    #[prop(optional, into)] base_form: Signal<Option<String>>,
    #[prop(optional, into)] grammar_label: Signal<Option<String>>,
    #[prop(optional, into)] grammar_description: Signal<Option<String>>,
    #[prop(optional, into)] translation_text: Signal<Option<String>>,
    /// #528+ quick-add: fires the deck-creation flow for the dictionary
    /// form; `None` when the repository context is down (isolated tests)
    /// or the token is not a vocabulary word — the button is not rendered.
    on_add_word: Option<Callback<()>>,
    on_report: Option<Callback<()>>,
) -> impl IntoView {
    let i18n = use_i18n();
    let popup_ref: NodeRef<leptos::html::Div> = NodeRef::new();
    let shift_x: RwSignal<f64> = RwSignal::new(0.0);

    Effect::new(move |_| {
        if let Some(el) = popup_ref.get() {
            let rect = el.get_bounding_client_rect();
            let vw = web_sys::window()
                .and_then(|w| w.inner_width().ok())
                .and_then(|v| v.as_f64())
                .unwrap_or(400.0);
            let margin = 8.0_f64;
            let popup_right = rect.right();
            let popup_left = rect.left();

            if popup_right > vw - margin {
                shift_x.set((vw - margin) - popup_right);
            } else if popup_left < margin {
                shift_x.set(margin - popup_left);
            }
        }
    });

    view! {
        <div
            class="token-popup"
            node_ref=popup_ref
            style=move || format!("--token-popup-shift: {}px", shift_x.get())
            on:click=move |ev: leptos::ev::MouseEvent| ev.stop_propagation()
        >
            <div class="token-popup-surface">{surface}</div>
            <div class="token-popup-reading">{reading}</div>
            {move || {
                base_form.get().map(|bf| view! {
                    <div class="token-popup-reading">{bf}</div>
                }.into_any()).unwrap_or_else(|| ().into_any())
            }}
            {move || {
                grammar_label.get().map(|label| view! {
                    <div class="token-popup-grammar">{label}</div>
                }.into_any()).unwrap_or_else(|| ().into_any())
            }}
            {move || {
                grammar_description.get().map(|desc| view! {
                    <div class="token-popup-grammar-description">{desc}</div>
                }.into_any()).unwrap_or_else(|| ().into_any())
            }}
            {move || {
                translation_text.get().map(|text| {
                    view! {
                        <MarkdownText
                            content=Signal::derive(move || text.clone())
                            known_kanji=HashSet::new()
                            variant=Signal::derive(|| MarkdownVariant::Compact)
                            furigana=false
                        />
                    }.into_any()
                }).unwrap_or_else(|| ().into_any())
            }}
            {move || {
                on_add_word.as_ref().map(|add| {
                    let add = *add;
                    view! {
                        <div class="token-popup-report-sep" aria-hidden="true"></div>
                        <button
                            class="token-popup-add"
                            data-testid="token-popup-add-word"
                            on:click=move |ev: leptos::ev::MouseEvent| {
                                ev.stop_propagation();
                                add.run(());
                            }
                        >
                            {t!(i18n, words.add_to_deck)}
                        </button>
                    }.into_any()
                }).unwrap_or_else(|| ().into_any())
            }}
            {move || {
                on_report.as_ref().map(|report| {
                    let report = *report;
                    view! {
                        <div class="token-popup-report-sep" aria-hidden="true"></div>
                        <button
                            class="token-popup-report"
                            data-testid="token-popup-report"
                            on:click=move |ev: leptos::ev::MouseEvent| {
                                ev.stop_propagation();
                                report.run(());
                            }
                        >
                            {t!(i18n, feedback.translator_report)}
                        </button>
                    }.into_any()
                }).unwrap_or_else(|| ().into_any())
            }}
        </div>
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use super::add_word_target;

    /// The quick-add target is the dictionary form (base); the surface
    /// fallback is a defensive guard — the tokenizer guarantees a
    /// non-empty base for vocabulary tokens.
    #[test]
    fn add_word_target_prefers_base_form() {
        assert_eq!(add_word_target("食べる", "食べた"), "食べる");
        assert_eq!(add_word_target("魚", "魚"), "魚");
    }

    #[test]
    fn add_word_target_falls_back_to_surface() {
        assert_eq!(add_word_target("", "食べた"), "食べた");
        assert_eq!(add_word_target("  ", "魚"), "魚");
    }
}
