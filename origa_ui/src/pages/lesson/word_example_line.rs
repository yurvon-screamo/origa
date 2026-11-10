//! Compact textbook example on a vocabulary card's answer side (#528).
//! Question sides never show it: the example would leak the translation.
//!
//! The sentence renders through the CONTEXT ruby (chunk pairs, Tanaka
//! readings) — 頭→あたま, not the lemma's dictionary entry とう. A ref
//! whose stored offsets are absent (kana-variant matches like the
//! auxiliary しまえ) renders unhighlighted and is only picked when no
//! surfacing ref exists for the word.

use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::loaders::example_loader::{WordExample, example_audio_path, load_word_examples};
use crate::ui_components::{
    AudioButtons, ExampleRubyText, HighlightRange, Text, TextSize, TypographyVariant,
};
use origa::domain::NativeLanguage;
use std::collections::HashSet;

/// A surfacing ref (the word really sits in the sentence) beats an
/// offset-less kana-variant match: the auxiliary てしまう usage of 仕舞う
/// must not pose as the lemma's example. When NO ref surfaces, the first
/// ref still renders (unhighlighted) — no example reads worse than a
/// word-less one for a card that has nothing else.
fn pick_surfaced_example(mut loaded: Vec<WordExample>) -> Option<WordExample> {
    let surfaced = loaded.iter().position(|we| we.start >= 0 && we.end >= 0);
    match surfaced {
        Some(pos) => Some(loaded.swap_remove(pos)),
        None => loaded.into_iter().next(),
    }
}

#[component]
pub fn WordExampleLine(
    word: String,
    /// Acquaintance-slide presentation: centered and larger. The answer
    /// side keeps the compact left-aligned default.
    #[prop(default = false)]
    prominent: bool,
    known_kanji: HashSet<char>,
    native_language: NativeLanguage,
) -> impl IntoView {
    let example: RwSignal<Option<WordExample>> = RwSignal::new(None);
    let word = StoredValue::new(word);
    let known = StoredValue::new(known_kanji);
    {
        spawn_local(async move {
            let loaded: Vec<WordExample> = load_word_examples(word.get_value().as_str())
                .await
                .into_iter()
                .flatten()
                .collect();
            if let Some(we) = pick_surfaced_example(loaded) {
                example.set(Some(we));
            }
        });
    }

    move || {
        let Some(we) = example.get() else {
            return ().into_any();
        };
        let WordExample { detail, start, end } = we;
        let sentence_text = detail.text.clone();
        let ruby = detail.ruby.clone();
        let word = word.get_value();
        // Stored offsets win; when absent (kana variant of a kanji word),
        // fall back to locating the surface form in the sentence. When
        // NEITHER locates the word, no highlight is drawn at all — an
        // empty highlight span reads as a stray tick mark.
        let highlight: HighlightRange = if start >= 0 && end >= 0 {
            (start, end)
        } else {
            match detail.text.find(&word) {
                Some(b) => {
                    let ci = detail.text[..b].chars().count();
                    (ci as i32, (ci + word.chars().count()) as i32)
                },
                None => (-1, -1),
            }
        };
        let translation = detail
            .translation(&native_language)
            .unwrap_or_default()
            .to_string();
        let known = known.get_value();
        let line_class = if prominent {
            "word-example-line word-example-line-prominent"
        } else {
            "word-example-line"
        };
        view! {
            <div class=line_class data-testid="lesson-word-example">
                // One line: the Japanese sentence + the play button at
                // the RIGHT EDGE of that line (owner UX — sound belongs
                // to the Japanese line, not to the translation).
                <div class="word-example-line-ja-line">
                    <div class="word-example-line-ja">
                        <ExampleRubyText
                            text=sentence_text.clone()
                            ruby=ruby
                            known_kanji=known
                            highlight=highlight
                        />
                    </div>
                    <AudioButtons
                        text=sentence_text
                        // #528: generated IndexTTS clip for this sentence
                        // (immutable CDN opus); TTS stays the fallback.
                        audio_path=Some(example_audio_path(detail.sentence_id))
                        test_id=Signal::derive(|| "lesson-word-example-audio".to_string())
                    />
                </div>
                // Translation centered under the sentence.
                <div class="word-example-line-translation">
                    <Text size=TextSize::Small variant=TypographyVariant::Muted>
                        {translation}
                    </Text>
                </div>
            </div>
        }
        .into_any()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use origa::dictionary::example::ExampleDetail;

    fn example(start: i32, end: i32, text: &str) -> WordExample {
        WordExample {
            detail: ExampleDetail {
                sentence_id: 0,
                text: text.to_string(),
                translation_en: None,
                translation_ru: None,
                translation_vi: None,
                translation_ko: None,
                ruby: vec![],
            },
            start,
            end,
        }
    }

    #[test]
    fn surfaced_ref_wins_over_kana_variant_match() {
        // The 仕舞う case: kana-variant refs (auxiliary てしまう) must not
        // pose as the lemma's example when a surfacing ref exists.
        let loaded = vec![
            example(-1, -1, "諦めてしまえ。"),
            example(2, 4, "頭を冷やせ。"),
        ];
        let picked = pick_surfaced_example(loaded).expect("a ref exists");
        assert_eq!(picked.detail.text, "頭を冷やせ。");
        assert_eq!((picked.start, picked.end), (2, 4));
    }

    #[test]
    fn word_less_refs_still_render_when_nothing_surfaces() {
        // No example reads worse than a word-less one for a card that has
        // nothing else: the first ref is the pick (unhighlighted render).
        let loaded = vec![
            example(-1, -1, "諦めてしまえ。"),
            example(-1, -1, "行ってしまえ！"),
        ];
        let picked = pick_surfaced_example(loaded).expect("a ref exists");
        assert_eq!(picked.detail.text, "諦めてしまえ。");
        assert_eq!((picked.start, picked.end), (-1, -1));
    }

    #[test]
    fn empty_ref_list_picks_nothing() {
        assert!(pick_surfaced_example(vec![]).is_none());
    }
}
