//! Word detail page (`/words/:word`): dictionary entry + textbook examples.
//! The word is a plain string (any dictionary word, not just the user's
//! cards), per the #528 boundary decision.

use std::collections::HashSet;

use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_router::hooks::use_params_map;

use crate::i18n::{locale_to_native_language, td_string, use_i18n};
use crate::loaders::example_loader::{WordExample, load_word_examples};
use crate::ui_components::{FuriganaText, Text, TextSize, TypographyVariant};
use origa::domain::NativeLanguage;

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

#[component]
pub fn WordDetail() -> impl IntoView {
    let i18n = use_i18n();
    let params = use_params_map();
    let word = move || params.get().get("word").unwrap_or_default();

    let native_lang = Memo::new(move |_| locale_to_native_language(&i18n.get_locale()));

    let known_kanji = Memo::new(move |_| HashSet::new());

    let translations = Memo::new(move |_| {
        let w = word();
        let lang = native_lang.get();
        origa::dictionary::vocabulary::get_translation(&w, &lang).unwrap_or_default()
    });

    let examples = RwSignal::new(Vec::<WordExample>::new());
    Effect::new(move |_| {
        let w = word();
        spawn_local(async move {
            let resolved: Vec<WordExample> =
                load_word_examples(&w).await.into_iter().flatten().collect();
            examples.set(resolved);
        });
    });

    let section_title =
        move || td_string!(i18n.get_locale(), words.detail_examples_section).to_string();
    let examples_empty =
        move || td_string!(i18n.get_locale(), words.detail_examples_not_found).to_string();

    view! {
        <div class="word-detail" data-testid="word-detail">
            <div class="word-detail-hero-card">
                <div class="word-detail-hero-word" data-testid="word-detail-word">
                    <FuriganaText text=word() known_kanji=known_kanji.get() test_id="word-detail-word-furi"/>
                </div>
                <div class="word-detail-hero-meaning" data-testid="word-detail-translation">
                    {move || translations.get()}
                </div>
            </div>

            <div class="word-detail-section">
                <div class="word-detail-section-title">{section_title}</div>
                {move || {
                    let details = examples.get();
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
                                                known_kanji=known_kanji.get()
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
        </div>
    }
}

#[component]
fn ExampleCard(
    example: WordExample,
    word: String,
    known_kanji: HashSet<char>,
    native_lang: NativeLanguage,
) -> impl IntoView {
    let WordExample { detail, start, end } = example;
    // Stored offsets win; when absent (kana variant), re-locate the surface.
    let (head, mid, tail) = if start >= 0 && end >= 0 {
        split_highlight(&detail.text, start, end)
    } else {
        let char_idx = detail
            .text
            .find(&word)
            .map(|b| detail.text[..b].chars().count());
        match char_idx {
            Some(ci) => {
                split_highlight(&detail.text, ci as i32, (ci + word.chars().count()) as i32)
            },
            None => split_highlight(&detail.text, -1, -1),
        }
    };

    let translation = detail
        .translation(&native_lang)
        .unwrap_or_default()
        .to_string();

    view! {
        <div class="word-detail-example-card" data-testid="word-detail-example">
            <div class="word-detail-example-ja">
                <FuriganaText text=head known_kanji=known_kanji.clone()/>
                {(!mid.is_empty()).then(|| {
                    view! {
                        <FuriganaText
                            text=mid
                            known_kanji=known_kanji.clone()
                            class=String::from("word-detail-example-highlight")
                        />
                    }
                })}
                <FuriganaText text=tail known_kanji=known_kanji/>
            </div>
            <div class="word-detail-example-translation" data-testid="word-detail-example-translation">
                {translation}
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
