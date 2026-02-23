use leptos::prelude::*;
use origa::domain::{furiganize_segments, FuriganaSegment};

#[component]
pub fn FuriganaText(
    #[prop(into)] text: String,
    #[prop(optional, into, default = String::new().into())] class: Signal<String>,
) -> impl IntoView {
    let segments = move || furiganize_segments(&text).unwrap_or_else(|_| vec![]);

    view! {
        <span class=move || format!("furigana-text {}", class.get())>
            <For
                each=segments
                key=|seg: &FuriganaSegment| (seg.text().to_string(), seg.reading().map(|r| r.to_string()))
                let:segment
            >
                {move || {
                    let text = segment.text().to_string();
                    let reading = segment.reading().map(|r| r.to_string());
                    render_segment(text, reading)
                }}
            </For>
        </span>
    }
}

fn render_segment(text: String, reading: Option<String>) -> impl IntoView {
    match reading {
        Some(reading) => view! {
            <ruby class="furigana-ruby">
                {text}
                <rp>"("</rp>
                <rt class="furigana-rt">{reading}</rt>
                <rp>")"</rp>
            </ruby>
        }
        .into_any(),
        None => view! {
            <span class="furigana-plain">{text}</span>
        }
        .into_any(),
    }
}
