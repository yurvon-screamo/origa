use std::collections::HashSet;

use leptos::prelude::*;
use origa::dictionary::kanji::get_kanji_info;
use origa::domain::{FuriganaSegment, JapaneseChar, NativeLanguage, furiganize_segments};

fn first_kanji_in_segment(segment_text: &str) -> Option<char> {
    segment_text.chars().find(|c| c.is_kanji())
}

fn extract_kanji_info(
    segment_text: &str,
    native_language: &NativeLanguage,
) -> Option<(char, String)> {
    let kanji = first_kanji_in_segment(segment_text)?;
    let info = get_kanji_info(&kanji.to_string()).ok()?;
    Some((kanji, info.description(native_language)))
}

#[component]
pub fn FuriganaTextWithHover(
    #[prop(into)] text: String,
    known_kanji: HashSet<char>,
    #[prop(optional, into)] native_language: Option<NativeLanguage>,
    #[prop(optional, into, default = String::new().into())] class: Signal<String>,
    #[prop(optional, into)] test_id: Signal<String>,
) -> impl IntoView {
    let segments = move || {
        furiganize_segments(&text, &known_kanji)
            .unwrap_or_else(|_| vec![FuriganaSegment::new(text.clone(), None, false)])
    };

    let test_id_val = move || {
        let val = test_id.get();
        if val.is_empty() { None } else { Some(val) }
    };

    let hovered: RwSignal<Option<usize>> = RwSignal::new(None);

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
                    let has_kanji = text.chars().any(|c| c.is_kanji());
                    render_segment_with_hover(
                        text,
                        reading,
                        is_known,
                        has_kanji,
                        hovered,
                        native_language,
                        tuple.0,
                    )
                }}
            </For>
        </span>
    }
}

fn render_segment_with_hover(
    text: String,
    reading: Option<String>,
    is_known: bool,
    has_kanji: bool,
    hovered: RwSignal<Option<usize>>,
    native_language: Option<NativeLanguage>,
    segment_id: usize,
) -> impl IntoView {
    if !has_kanji || native_language.is_none() {
        return render_segment(text, reading, is_known).into_any();
    }

    let lang = native_language.expect("checked above");
    let kanji_info = extract_kanji_info(&text, &lang);

    let Some((kanji_char, description)) = kanji_info else {
        return render_segment(text, reading, is_known).into_any();
    };

    let kanji_display = kanji_char.to_string();
    let desc_display = description;

    let base_view = render_segment(text, reading, is_known);

    let enter_handler = move |ev: leptos::ev::MouseEvent| {
        let _ = ev;
        hovered.set(Some(segment_id));
    };
    let leave_handler = move |_| {
        hovered.set(None);
    };
    let click_handler = move |ev: leptos::ev::MouseEvent| {
        ev.stop_propagation();
        hovered.update(|h| {
            *h = if *h == Some(segment_id) {
                None
            } else {
                Some(segment_id)
            };
        });
    };

    let is_active = move || hovered.get() == Some(segment_id);

    view! {
        <span
            class="kanji-hover-segment"
            on:mouseenter=enter_handler
            on:mouseleave=leave_handler
        >
            <span
                class=move || {
                    if is_active() {
                        "kanji-hover-trigger kanji-hover-active"
                    } else {
                        "kanji-hover-trigger"
                    }
                }
                on:click=click_handler
            >
                {base_view}
            </span>
            <Show when=is_active>
                <div class="kanji-popup" on:click=move |ev: leptos::ev::MouseEvent| ev.stop_propagation()>
                    <div class="kanji-popup-char font-serif">{kanji_display.clone()}</div>
                    <div class="kanji-popup-desc">{desc_display.clone()}</div>
                </div>
            </Show>
        </span>
    }
    .into_any()
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
