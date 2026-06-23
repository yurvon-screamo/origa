use std::collections::HashSet;

use leptos::prelude::*;
use origa::domain::{FuriganaSegment, NativeLanguage, furiganize_segments};

use crate::ui_components::furigana_hover::render_segment_with_hover;

#[component]
pub fn FuriganaText(
    #[prop(into)] text: String,
    known_kanji: HashSet<char>,
    #[prop(optional, into, default = String::new().into())] class: Signal<String>,
    #[prop(optional, into)] test_id: Signal<String>,
    #[prop(optional, into)] native_language: Option<NativeLanguage>,
    #[prop(optional, default = false)] with_kanji_tooltip: bool,
) -> impl IntoView {
    let segments = move || {
        furiganize_segments(&text, &known_kanji)
            .unwrap_or_else(|_| vec![FuriganaSegment::new(text.clone(), None, false)])
    };

    let test_id_val = move || {
        let val = test_id.get();
        if val.is_empty() { None } else { Some(val) }
    };

    let tooltip_enabled = with_kanji_tooltip && native_language.is_some();
    let hovered: RwSignal<Option<(usize, usize)>> = RwSignal::new(None);

    view! {
        <span
            class=move || format!("furigana-text {}", class.get())
            data-testid=test_id_val
        >
            <For
                each=move || {
                    segments().into_iter().enumerate().collect::<Vec<_>>()
                }
                key=|(idx, seg): &(usize, FuriganaSegment)| {
                    (*idx, seg.text().to_string(), seg.reading().map(|r| r.to_string()))
                }
                let:tuple
            >
                {move || {
                    let text = tuple.1.text().to_string();
                    let reading = tuple.1.reading().map(|r| r.to_string());
                    let is_known = tuple.1.is_known();
                    if tooltip_enabled {
                        render_segment_with_hover(
                            text,
                            reading,
                            is_known,
                            native_language,
                            tuple.0,
                            hovered,
                        )
                    } else {
                        render_segment(text, reading, is_known).into_any()
                    }
                }}
            </For>
        </span>
    }
}

fn render_segment(text: String, reading: Option<String>, is_known: bool) -> impl IntoView {
    match reading {
        Some(reading) => {
            let class = if is_known {
                "furigana-hidden furigana-ruby anima-furigana-show"
            } else {
                "furigana-ruby anima-furigana-show"
            };
            view! {
                <ruby class=class>
                    {text}
                    <rp>"("</rp>
                    <rt class="furigana-rt">{reading}</rt>
                    <rp>")"</rp>
                </ruby>
            }
            .into_any()
        },
        None => view! {
            <span class="furigana-plain">{text}</span>
        }
        .into_any(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn render_segment_with_reading_returns_ruby() {
        let _ = render_segment("食べ".to_string(), Some("たべ".to_string()), false);
    }

    #[test]
    fn render_segment_without_reading_returns_plain() {
        let _ = render_segment("たべ".to_string(), None, false);
    }
}
