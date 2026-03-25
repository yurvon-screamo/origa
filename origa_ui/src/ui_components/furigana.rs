use std::collections::HashSet;

use leptos::prelude::*;
use origa::domain::{FuriganaSegment, furiganize_segments};

#[component]
pub fn FuriganaText(
    #[prop(into)] text: String,
    known_kanji: HashSet<String>,
    #[prop(optional, into, default = String::new().into())] class: Signal<String>,
    #[prop(optional, into)] test_id: Signal<String>,
) -> impl IntoView {
    let segments = move || furiganize_segments(&text, &known_kanji).unwrap_or_else(|_| vec![]);

    let test_id_val = move || {
        let val = test_id.get();
        if val.is_empty() { None } else { Some(val) }
    };

    view! {
        <span
            class=move || format!("furigana-text {}", class.get())
            data-testid=test_id_val
        >
            <For
                each=segments
                key=|seg: &FuriganaSegment| (seg.text().to_string(), seg.reading().map(|r| r.to_string()))
                let:segment
            >
                {move || {
                    let text = segment.text().to_string();
                    let reading = segment.reading().map(|r| r.to_string());
                    let is_known = segment.is_known();
                    render_segment(text, reading, is_known)
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
