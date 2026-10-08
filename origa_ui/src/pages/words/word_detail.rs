//! Word detail page (`/words/:word`): dictionary entry + textbook examples.
//! The word is a plain string (any dictionary word, not just the user's
//! cards), per the #528 boundary decision. When the user also has a study
//! card for the word, the FSRS metrics and card actions render above.

use std::collections::HashSet;

use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_router::components::A;
use leptos_router::hooks::{use_navigate, use_params_map};

use crate::i18n::{locale_to_native_language, td_string, use_i18n};
use crate::loaders::example_loader::{WordExample, load_word_examples};
use crate::pages::shared::{
    CardStatus, DeleteRequest, create_delete_callback, create_mark_as_known_callback,
};
use crate::repository::HybridUserRepository;
use crate::ui_components::{
    AudioButtons, CardActionBar, DeleteConfirmModal, FsrsMetrics, FuriganaText, Text, TextSize,
    TranslatorText, TypographyVariant,
};
use origa::domain::{Card as DomainCard, NativeLanguage, StudyCard, User};
use origa::traits::UserRepository;
use origa::use_cases::ToggleFavoriteUseCase;
use ulid::Ulid;

/// Split a sentence at the highlighted word occurrence. Character offsets;
/// negative offsets mean "unlocatable" — everything stays unhighlighted.
pub fn split_highlight(text: &str, start: i32, end: i32) -> (String, String, String) {
    if start < 0 || end < 0 {
        return (text.to_string(), String::new(), String::new());
    }
    let (start, end) = (start as usize, end as usize);
    let chars: Vec<char> = text.chars().collect();
    if start >= end || start > chars.len() || end > chars.len() {
        return (text.to_string(), String::new(), String::new());
    }
    let head: String = chars[..start].iter().collect();
    let mid: String = chars[start..end].iter().collect();
    let tail: String = chars[end..].iter().collect();
    (head, mid, tail)
}

/// Find the user's vocabulary study card for a dictionary word.
fn find_word_card(user: &User, word: &str) -> Option<(Ulid, StudyCard)> {
    let found = user
        .knowledge_set()
        .study_cards()
        .iter()
        .find_map(|(id, card)| match card.card() {
            DomainCard::Vocabulary(v) if v.word().text() == word => Some((*id, card.clone())),
            _ => None,
        });
    if found.is_none() {
        // #528 owner report: the action bar was missing on a card the
        // owner considers theirs. Log the exact mismatch for diagnosis:
        // is the URL word form different from the stored card word?
        let vocab_total = user
            .knowledge_set()
            .study_cards()
            .values()
            .filter(|c| matches!(c.card(), DomainCard::Vocabulary(_)))
            .count();
        tracing::warn!(
            word,
            vocab_total,
            "Word detail: no card for this word — action bar hidden"
        );
    }
    found
}

#[component]
pub fn WordDetail() -> impl IntoView {
    let i18n = use_i18n();
    let repository =
        use_context::<HybridUserRepository>().expect("repository context not provided");

    let params = use_params_map();
    let word = move || params.get().get("word").unwrap_or_default();

    let current_user: RwSignal<Option<User>> = RwSignal::new(None);
    let study_card: RwSignal<Option<StudyCard>> = RwSignal::new(None);
    let refresh_trigger = RwSignal::new(0u32);
    let is_delete_modal_open = RwSignal::new(false);
    let repo_for_effect = repository.clone();
    let card_generation = StoredValue::new(0u64);

    // Resolve the user's card for this word (may legitimately be absent —
    // the page is a dictionary reference for any word). Generation-guard
    // mirrors the examples effect: fast navigation must not let word A's
    // card render under word B.
    Effect::new(move |_| {
        let _ = refresh_trigger.get();
        is_delete_modal_open.set(false);
        let w = word();
        card_generation.update_value(|g| *g += 1);
        let generation = card_generation.get_value();
        let repo = repo_for_effect.clone();
        spawn_local(async move {
            let outcome = match repo.get_current_user().await {
                Ok(Some(user)) => {
                    let found = find_word_card(&user, &w);
                    (Some(user), found.map(|(_, card)| card))
                },
                _ => (None, None),
            };
            if card_generation.get_value() == generation {
                current_user.set(outcome.0);
                study_card.set(outcome.1);
            }
        });
    });

    let native_lang = Memo::new(move |_| locale_to_native_language(&i18n.get_locale()));

    // All dictionary translations + description for the current word.
    let translations = Memo::new(move |_| {
        let w = word();
        let lang = native_lang.get();
        origa::dictionary::vocabulary::get_translations(&w, &lang).unwrap_or_default()
    });
    let description = Memo::new(move |_| {
        let w = word();
        let lang = native_lang.get();
        origa::dictionary::vocabulary::get_description(&w, &lang).filter(|s| !s.is_empty())
    });

    // Loaded examples for the CURRENT word. `None` = fetch in flight
    // (render nothing: an empty state would flash a false "no examples").
    let examples: RwSignal<Option<Vec<WordExample>>> = RwSignal::new(None);
    let example_generation = StoredValue::new(0u64);
    Effect::new(move |_| {
        let w = word();
        // The route reuses this component across /words/A -> /words/B:
        // drop the previous word's list immediately and tag the in-flight
        // fetch, so a slow stale response can never overwrite a newer one.
        example_generation.update_value(|g| *g += 1);
        let generation = example_generation.get_value();
        examples.set(None);
        spawn_local(async move {
            let resolved: Vec<WordExample> =
                load_word_examples(&w).await.into_iter().flatten().collect();
            if example_generation.get_value() == generation {
                examples.set(Some(resolved));
            }
        });
    });

    let section_words_title = move || td_string!(i18n.get_locale(), words.header).to_string();
    let examples_empty =
        move || td_string!(i18n.get_locale(), words.detail_examples_not_found).to_string();
    let breadcrumbs_label = section_words_title;

    // ── Card actions (present only when the user studies this word) ──
    let is_favorite_signal: RwSignal<bool> = RwSignal::new(false);
    Effect::new(move |_| {
        if let Some(card) = study_card.get() {
            is_favorite_signal.set(card.is_favorite());
        }
    });
    let favorite_pending = RwSignal::new(false);
    let on_toggle_favorite = {
        let repo = repository.clone();
        let current_user_fav = current_user;
        let refresh = refresh_trigger;
        let pending = favorite_pending;
        Callback::new(move |card_id: Ulid| {
            is_favorite_signal.update(|f| *f = !*f);
            let repo = repo.clone();
            spawn_local(async move {
                pending.set(true);
                let use_case = ToggleFavoriteUseCase::new(&repo);
                if use_case.execute(card_id).await.is_ok() {
                    current_user_fav.update(|u| {
                        if let Some(user) = u {
                            let _ = user.toggle_favorite(card_id);
                        }
                    });
                    refresh.update(|v| *v += 1);
                } else {
                    is_favorite_signal.update(|f| *f = !*f);
                }
                pending.set(false);
            });
        })
    };
    let (on_mark_as_known, mark_known_pending) = {
        let repo = repository.clone();
        create_mark_as_known_callback(repo, refresh_trigger)
    };
    let toasts: RwSignal<Vec<crate::ui_components::ToastData>> = RwSignal::new(Vec::new());
    let (is_deleting, on_delete) =
        create_delete_callback(repository.clone(), toasts, refresh_trigger);
    let navigate = StoredValue::new(use_navigate());

    let hero_word = word;
    let hero_view = move || {
        // Re-rendered per word: the route component survives navigation
        // between /words/A and /words/B, so a static snapshot would leave
        // the previous word in the hero. The page is reference material:
        // the reading shows over every kanji (empty known set), matching
        // the examples below.
        let w = hero_word();
        view! {
            <FuriganaText text=w known_kanji=HashSet::new() test_id="word-detail-word-furi"/>
        }
        .into_any()
    };

    view! {
        <div class="word-detail-container" data-testid="word-detail">
            // Breadcrumbs: Words / <word>
            <div class="kanji-detail-top-bar">
                <div class="kanji-breadcrumbs" data-testid="word-detail-breadcrumbs">
                    <A href="/words">{breadcrumbs_label}</A>
                    <span class="kanji-breadcrumbs-separator">"/"</span>
                    <span class="kanji-breadcrumbs-current">{move || word()}</span>
                </div>
                <Show when=move || study_card.get().is_some()>
                    {move || {
                        let card = study_card.get().expect("checked by Show");
                        let memory = card.memory().clone();
                        let status = CardStatus::from_study_card(&card);
                        let card_id = *card.card_id();
                        view! {
                            <FsrsMetrics
                                difficulty=memory.difficulty().map(|d| d.value())
                                stability=memory.stability().map(|s| s.value())
                                test_id=Signal::derive(|| "word-detail-fsrs".to_string())
                            />
                            <CardActionBar
                                tag_variant=Signal::derive(move || status.tag_variant())
                                tag_label=Signal::derive(move || status.label(&i18n))
                                is_favorite=is_favorite_signal.into()
                                on_toggle_favorite=Callback::new(move |_| on_toggle_favorite.run(card_id))
                                favorite_pending=favorite_pending
                                show_mark_as_known=Signal::derive(move || status != CardStatus::Learned)
                                on_mark_as_known=Callback::new(move |_| on_mark_as_known.run(card_id))
                                mark_known_pending=mark_known_pending
                                on_delete=Callback::new(move |_| is_delete_modal_open.set(true))
                                test_id=Signal::derive(|| "word-detail-actions".to_string())
                                show_tag=Signal::derive(|| false)
                            />
                        }
                    }}
                </Show>
            </div>

            <div class="word-detail-hero-card">
                <div class="word-detail-hero-word" data-testid="word-detail-word">
                    {hero_view}
                </div>
            </div>

            <div class="word-detail-section">
                <div class="word-detail-section-title">
                    {move || td_string!(i18n.get_locale(), words.detail_translation_section).to_string()}
                </div>
                <div class="word-detail-translations-card" data-testid="word-detail-translation">
                    <For
                        each=move || translations.get()
                        key=|t: &String| t.clone()
                        children=move |t: String| {
                            view! {
                                <div class="word-detail-translation-row">
                                    <Text size=TextSize::Default variant=TypographyVariant::Primary>
                                        {t}
                                    </Text>
                                </div>
                            }
                        }
                    />
                    <Show when=move || description.get().is_some()>
                        <div class="word-detail-description">
                            <Text size=TextSize::Small variant=TypographyVariant::Muted>
                                {move || description.get().unwrap_or_default()}
                            </Text>
                        </div>
                    </Show>
                </div>
            </div>

            <div class="word-detail-section">
                <div class="word-detail-section-title">
                    {move || td_string!(i18n.get_locale(), words.detail_examples_section).to_string()}
                </div>
                {move || {
                    let details = examples.get();
                    let Some(details) = details else {
                        // Fetch in flight: render nothing instead of a false
                        // "no examples yet" empty state.
                        return ().into_any();
                    };
                    if details.is_empty() {
                        view! {
                            <Text size=TextSize::Default variant=TypographyVariant::Muted>
                                {examples_empty()}
                            </Text>
                        }.into_any()
                    } else {
                        view! {
                            <div class="word-detail-examples" data-testid="word-detail-examples">
                                <For
                                    each=move || details.clone()
                                    key=|we: &WordExample| we.detail.sentence_id
                                    children=move |we| {
                                        view! {
                                            <ExampleCard
                                                example=we
                                                word=word()
                                                native_lang=native_lang.get()
                                            />
                                        }
                                    }
                                />
                            </div>
                        }.into_any()
                    }
                }}
            </div>

            <Show when=move || is_delete_modal_open.get() && study_card.get().is_some()>
                {move || {
                    // Single source of truth: the card comes from the same
                    // signal the Show guard reads — no re-derivation from
                    // the word param, which can mismatch mid-navigation
                    // (a derived None here used to be a reachable WASM
                    // panic).
                    let Some(card) = study_card.get() else {
                        return ().into_any();
                    };
                    let card_id = *card.card_id();
                    let confirm_delete = Callback::new(move |_| {
                        on_delete.run(DeleteRequest {
                            card_id,
                            on_success: Callback::new(move |_| {
                                is_delete_modal_open.set(false);
                                navigate.get_value()("/words", Default::default());
                            }),
                        })
                    });
                    view! {
                        <DeleteConfirmModal
                            is_open=is_delete_modal_open
                            is_deleting=is_deleting.into()
                            on_confirm=confirm_delete
                            on_close=Callback::new(move |_| is_delete_modal_open.set(false))
                        />
                    }
                    .into_any()
                }}
            </Show>
        </div>
    }
}

#[component]
fn ExampleCard(example: WordExample, word: String, native_lang: NativeLanguage) -> impl IntoView {
    let WordExample { detail, start, end } = example;
    let _ = (&start, &end, &word); // offsets unused by the token translator

    let translation = detail
        .translation(&native_lang)
        .unwrap_or_default()
        .to_string();

    // The word page is reference material — the sentence renders through
    // the TOKEN TRANSLATOR (the phrase-card etalon, owner request): every
    // token is interactive with its own translation. The word highlight
    // and the furigana segments are gone: the hero word sits right above.
    let sentence = detail.text.clone();

    view! {
        <div class="word-detail-example-card" data-testid="word-detail-example">
            <div class="word-detail-example-ja">
                <div class="word-detail-example-ja-row">
                    <TranslatorText
                        text=sentence
                        class=Signal::derive(|| "text-2xl leading-relaxed".to_string())
                        test_id=Signal::derive(|| "word-detail-example-sentence".to_string())
                    />
                    // The play button sits next to THE SENTENCE, not to
                    // the translation (owner UX).
                    <AudioButtons
                        text=detail.text
                        audio_path=None
                        test_id=Signal::derive(|| "word-detail-example-audio".to_string())
                    />
                </div>
                <div
                    class="word-detail-example-translation"
                    data-testid="word-detail-example-translation"
                >
                    {translation}
                </div>
            </div>
        </div>
    }
}

#[cfg(test)]
mod tests {
    use super::split_highlight;

    #[test]
    fn splits_at_char_offsets() {
        let (h, m, t) = split_highlight("家族は４人です。", 0, 2);
        assert_eq!(h, "");
        assert_eq!(m, "家族");
        assert_eq!(t, "は４人です。");
    }

    #[test]
    fn middle_occurrence_splits_in_three() {
        let (h, m, t) = split_highlight("彼は家族を捨てた。", 2, 4);
        assert_eq!(h, "彼は");
        assert_eq!(m, "家族");
        assert_eq!(t, "を捨てた。");
    }

    #[test]
    fn negative_offsets_highlight_nothing() {
        let (h, m, t) = split_highlight("どっちでも同じ。", -1, -1);
        assert_eq!(m, "");
        assert_eq!(h + &m + &t, "どっちでも同じ。");
    }

    #[test]
    fn out_of_range_offsets_highlight_nothing() {
        let (h, m, t) = split_highlight("短い", 0, 99);
        assert_eq!(m, "");
        assert_eq!(h + &m + &t, "短い");
        let (h, m, t) = split_highlight("短い", 5, 9);
        assert_eq!(m, "");
        assert_eq!(h + &m + &t, "短い");
    }

    #[test]
    fn multibyte_text_is_char_safe() {
        // 6 chars, 18 bytes: byte-based slicing would panic or split glyphs.
        let (h, m, t) = split_highlight("こんにちは世界", 3, 5);
        assert_eq!(h, "こんに");
        assert_eq!(m, "ちは");
        assert_eq!(t, "世界");
    }
}
