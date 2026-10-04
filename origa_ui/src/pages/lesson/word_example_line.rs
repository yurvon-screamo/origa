//! Compact textbook example on a vocabulary card's answer side (#528).
//! Question sides never show it: the example would leak the translation.

use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::loaders::example_loader::{WordExample, load_word_examples};
use crate::ui_components::{FuriganaText, Text, TextSize, TypographyVariant};
use origa::domain::NativeLanguage;
use std::collections::HashSet;

#[component]
pub fn WordExampleLine(
    word: String,
    known_kanji: HashSet<char>,
    native_language: NativeLanguage,
) -> impl IntoView {
    let example = RwSignal::new(None);
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

    view! {
        <Show when=move || example.get().is_some()>
            {move || {
                let we = example.get().expect("checked by Show");
                let word = word.get_value();
                let WordExample { detail, start, end } = we;
                let (head, mid, tail) = if start >= 0 && end >= 0 {
                    let chars: Vec<char> = detail.text.chars().collect();
                    let (s, e) = (start as usize, end as usize);
                    if s <= e && e <= chars.len() {
                        (
                            chars[..s].iter().collect::<String>(),
                            chars[s..e].iter().collect::<String>(),
                            chars[e..].iter().collect::<String>(),
                        )
                    } else {
                        (detail.text.clone(), String::new(), String::new())
                    }
                } else {
                    match detail.text.find(&word) {
                        Some(b) => {
                            let ci = detail.text[..b].chars().count();
                            let ce = ci + word.chars().count();
                            let chars: Vec<char> = detail.text.chars().collect();
                            (
                                chars[..ci].iter().collect::<String>(),
                                chars[ci..ce].iter().collect::<String>(),
                                chars[ce..].iter().collect::<String>(),
                            )
                        },
                        None => (detail.text.clone(), String::new(), String::new()),
                    }
                };
                let translation = detail
                    .translation(&native_language)
                    .unwrap_or_default()
                    .to_string();
                view! {
                    <div class="word-example-line" data-testid="lesson-word-example">
                        <div class="word-example-line-ja">
                            <FuriganaText text=head.clone() known_kanji=known.get_value()/>
                            <FuriganaText
                                text=mid.clone()
                                known_kanji=known.get_value()
                                class=String::from("word-detail-example-highlight")
                            />
                            <FuriganaText text=tail.clone() known_kanji=known.get_value()/>
                        </div>
                        <Text size=TextSize::Small variant=TypographyVariant::Muted>
                            {translation}
                        </Text>
                    </div>
                }
            }}
        </Show>
    }
}
