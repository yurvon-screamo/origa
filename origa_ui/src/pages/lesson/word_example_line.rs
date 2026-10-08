//! Compact textbook example on a vocabulary card's answer side (#528).
//! Question sides never show it: the example would leak the translation.

use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::loaders::example_loader::{WordExample, load_word_examples};
use crate::pages::words::split_highlight;
use crate::ui_components::{AudioButtons, FuriganaText, Text, TextSize, TypographyVariant};
use origa::domain::NativeLanguage;
use std::collections::HashSet;

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
            let loaded = load_word_examples(word.get_value().as_str()).await;
            if let Some(Ok(we)) = loaded.into_iter().next() {
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
        let word = word.get_value();
        // Stored offsets win; when absent (kana variant of a kanji word),
        // fall back to locating the surface form in the sentence. When
        // NEITHER locates the word (mid empty), render one unhighlighted
        // run — an empty highlight span reads as a stray tick mark.
        let (head, mid, tail) = if start >= 0 && end >= 0 {
            split_highlight(&detail.text, start, end)
        } else {
            match detail.text.find(&word) {
                Some(b) => {
                    let ci = detail.text[..b].chars().count();
                    split_highlight(&detail.text, ci as i32, (ci + word.chars().count()) as i32)
                },
                None => split_highlight(&detail.text, -1, -1),
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
                // The play button sits next to THE SENTENCE (owner UX:
                // sound belongs to the Japanese line, not to the
                // translation).
                <div class="word-example-line-ja-row">
                    <AudioButtons
                        text=sentence_text
                        audio_path=None
                        test_id=Signal::derive(|| "lesson-word-example-audio".to_string())
                    />
                </div>
                <div class="word-example-line-ja">
                    {if mid.is_empty() {
                        // No highlight to draw: one run.
                        view! {
                            <FuriganaText text=head known_kanji=known/>
                        }
                        .into_any()
                    } else {
                        view! {
                            <FuriganaText text=head.clone() known_kanji=known.clone()/>
                            <FuriganaText
                                text=mid
                                known_kanji=known.clone()
                                class=String::from("word-example-highlight")
                            />
                            <FuriganaText text=tail known_kanji=known/>
                        }
                        .into_any()
                    }}
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
