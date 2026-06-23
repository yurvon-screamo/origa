use leptos::prelude::*;
use origa::dictionary::kanji::get_kanji_info;
use origa::domain::{JapaneseChar, NativeLanguage};

#[derive(Clone, Copy, PartialEq, Debug)]
pub(crate) enum SegmentCharKind {
    Kanji,
    Other,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub(crate) struct SegmentChar {
    pub(crate) ch: char,
    pub(crate) kind: SegmentCharKind,
}

pub(crate) fn split_segment_to_chars(text: &str) -> Vec<SegmentChar> {
    text.chars()
        .map(|ch| {
            let kind = if ch.is_kanji() {
                SegmentCharKind::Kanji
            } else {
                SegmentCharKind::Other
            };
            SegmentChar { ch, kind }
        })
        .collect()
}

fn segment_has_kanji(text: &str) -> bool {
    text.chars().any(|c| c.is_kanji())
}

pub(crate) fn render_segment_with_hover(
    text: String,
    reading: Option<String>,
    is_known: bool,
    native_language: Option<NativeLanguage>,
    segment_id: usize,
    hovered: RwSignal<Option<(usize, usize)>>,
) -> AnyView {
    let Some(lang) = native_language else {
        return render_plain_segment(text, reading, is_known).into_any();
    };
    if !segment_has_kanji(&text) {
        return render_plain_segment(text, reading, is_known).into_any();
    }

    let chars = split_segment_to_chars(&text);
    let char_descs: Vec<Option<String>> = chars
        .iter()
        .map(|sc| {
            if sc.kind == SegmentCharKind::Kanji {
                get_kanji_info(&sc.ch.to_string())
                    .ok()
                    .map(|info| info.description(&lang))
            } else {
                None
            }
        })
        .collect();

    let has_any_tooltip = char_descs.iter().any(|d| d.is_some());
    if !has_any_tooltip {
        return render_plain_segment(text, reading, is_known).into_any();
    }

    let reading_view = reading.map(|r| {
        view! {
            <rp>"("</rp>
            <rt class="furigana-rt">{r}</rt>
            <rp>")"</rp>
        }
        .into_any()
    });

    let class = if is_known {
        "furigana-hidden furigana-ruby anima-furigana-show"
    } else {
        "furigana-ruby anima-furigana-show"
    };

    let char_descs_stored = StoredValue::new(char_descs);
    let chars_stored = StoredValue::new(chars);

    view! {
        <ruby class=class>
            <For
                each=move || {
                    chars_stored
                        .get_value()
                        .iter()
                        .copied()
                        .enumerate()
                        .collect::<Vec<_>>()
                }
                key=|(idx, sc): &(usize, SegmentChar)| (*idx, sc.ch)
                let:pair
            >
                {move || {
                    let (char_idx, sc) = pair;
                    let desc = char_descs_stored.get_value()[char_idx].clone();
                    let ch = sc.ch;
                    let key = (segment_id, char_idx);
                    let is_active = move || hovered.get() == Some(key);
                    let enter = move |_| hovered.set(Some(key));
                    let leave = move |_| {
                        if hovered.get() == Some(key) {
                            hovered.set(None);
                        }
                    };
                    let click = move |ev: leptos::ev::MouseEvent| {
                        ev.stop_propagation();
                        hovered.update(|h| {
                            *h = if *h == Some(key) { None } else { Some(key) };
                        });
                    };
                    let desc_text = desc.clone().unwrap_or_default();
                    let has_desc = desc.is_some();
                    view! {
                        <span
                            class=move || {
                                if sc.kind == SegmentCharKind::Kanji && is_active() {
                                    "kanji-char-hover kanji-hover-active"
                                } else if sc.kind == SegmentCharKind::Kanji {
                                    "kanji-char-hover"
                                } else {
                                    ""
                                }
                            }
                            on:mouseenter=enter
                            on:mouseleave=leave
                            on:click=click
                        >
                            {ch}
                            <Show when=move || sc.kind == SegmentCharKind::Kanji && has_desc && is_active()>
                                <div class="kanji-popup" on:click=move |ev: leptos::ev::MouseEvent| ev.stop_propagation()>
                                    <div class="kanji-popup-char font-serif">{ch}</div>
                                    <div class="kanji-popup-desc">{desc_text.clone()}</div>
                                </div>
                            </Show>
                        </span>
                    }
                }}
            </For>
            {reading_view}
        </ruby>
    }
    .into_any()
}

pub(crate) fn render_plain_segment(text: String, reading: Option<String>, is_known: bool) -> impl IntoView {
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
    fn splits_multi_kanji_segment_into_kanji_chars() {
        let chars = split_segment_to_chars("木綿");
        assert_eq!(chars.len(), 2);
        assert_eq!(chars[0].ch, '木');
        assert_eq!(chars[0].kind, SegmentCharKind::Kanji);
        assert_eq!(chars[1].ch, '綿');
        assert_eq!(chars[1].kind, SegmentCharKind::Kanji);
    }

    #[test]
    fn splits_hiragana_segment_as_all_other() {
        let chars = split_segment_to_chars("こちら");
        assert_eq!(chars.len(), 3);
        for sc in &chars {
            assert_eq!(sc.kind, SegmentCharKind::Other);
        }
        assert_eq!(chars[0].ch, 'こ');
        assert_eq!(chars[1].ch, 'ち');
        assert_eq!(chars[2].ch, 'ら');
    }

    #[test]
    fn splits_mixed_kanji_kana_segment_correctly() {
        let chars = split_segment_to_chars("お木");
        assert_eq!(chars.len(), 2);
        assert_eq!(chars[0].ch, 'お');
        assert_eq!(chars[0].kind, SegmentCharKind::Other);
        assert_eq!(chars[1].ch, '木');
        assert_eq!(chars[1].kind, SegmentCharKind::Kanji);
    }

    #[test]
    fn splits_empty_segment_to_empty_vec() {
        let chars = split_segment_to_chars("");
        assert!(chars.is_empty());
    }

    #[test]
    fn splits_latin_chars_as_other() {
        let chars = split_segment_to_chars("abc");
        assert_eq!(chars.len(), 3);
        for sc in &chars {
            assert_eq!(sc.kind, SegmentCharKind::Other);
        }
    }

    #[test]
    fn splits_single_kanji_segment() {
        let chars = split_segment_to_chars("猫");
        assert_eq!(chars.len(), 1);
        assert_eq!(chars[0].ch, '猫');
        assert_eq!(chars[0].kind, SegmentCharKind::Kanji);
    }
}
