//! Furigana render for sentence EXAMPLES: the chunk's context ruby pairs
//! (Tanaka B-line readings) take priority over the dictionary, so 頭 in
//! 「頭を冷やせ。」 shows あたま instead of the lemma's first dictionary
//! entry とう. Sits on [`furiganize_segments_with_ruby`] (domain), falls
//! back to the plain dictionary path when the sentence has no usable pairs.

use std::collections::HashSet;

use leptos::prelude::*;
use origa::dictionary::example::RubyPair;
use origa::domain::{FuriganaSegment, furigana_context::furiganize_segments_with_ruby};

use crate::store::auth_store::AuthStore;

/// Char offsets (inclusive start, exclusive end) of the card-word
/// occurrence the example highlights; segments overlapping the range get
/// the highlight class.
pub type HighlightRange = (i32, i32);

/// Whether the segment occupying char range
/// `[offset_before, offset_before + segment_len)` intersects the
/// highlight word range. Negative highlight bounds mean "no word to
/// highlight" — no segment ever matches.
pub fn segment_overlaps_highlight(
    offset_before: usize,
    segment_len: usize,
    highlight: HighlightRange,
) -> bool {
    let (start, end) = highlight;
    if start < 0 || end < 0 {
        return false;
    }
    let (start, end) = (start as usize, end as usize);
    let segment_end = offset_before + segment_len;
    segment_end > start && offset_before < end
}

#[component]
pub fn ExampleRubyText(
    text: String,
    ruby: Vec<RubyPair>,
    known_kanji: HashSet<char>,
    #[prop(into, default = (-1, -1))] highlight: HighlightRange,
    #[prop(optional, into, default = String::new().into())] class: Signal<String>,
) -> impl IntoView {
    // Miss paths (no usable context pair) depend on the tokenizer
    // dictionary, which left the startup overlay (#521) — same readiness
    // tracking as FuriganaText.
    let tokenizer_ready = use_context::<AuthStore>().map(|store| store.is_dictionary_loaded);

    let segments = move || {
        if let Some(ready) = tokenizer_ready {
            ready.get();
        }
        let known = known_kanji.clone();
        match furiganize_segments_with_ruby(&text, &ruby, &known) {
            Ok(segments) if !segments.is_empty() => segments,
            _ => {
                // FuriganaError (dictionary not loaded yet) or an empty
                // derivation: one plain run, same as FuriganaText's fallback.
                vec![FuriganaSegment::new(text.clone(), None, false)]
            },
        }
    };

    let is_highlighted = move |segment: &FuriganaSegment, offset_before: usize| {
        segment_overlaps_highlight(offset_before, segment.text().chars().count(), highlight)
    };

    view! {
        <span class=move || format!("furigana-text {}", class.get())>
            {move || {
                let mut offset_before = 0usize;
                segments()
                    .into_iter()
                    .map(|segment| {
                        let highlighted = is_highlighted(&segment, offset_before);
                        offset_before += segment.text().chars().count();
                        render_segment(segment, highlighted)
                    })
                    .collect_view()
            }}
        </span>
    }
}

fn render_segment(segment: FuriganaSegment, highlighted: bool) -> impl IntoView {
    let text = segment.text().to_string();
    let class = if highlighted {
        "word-example-highlight"
    } else {
        ""
    };
    match segment.reading() {
        Some(reading) => {
            let ruby_class = if segment.is_known() {
                "furigana-hidden furigana-ruby"
            } else {
                "furigana-ruby"
            };
            view! {
                <span class=class>
                    <ruby class=ruby_class>
                        {text}
                        <rp>"("</rp>
                        <rt class="furigana-rt">{reading}</rt>
                        <rp>")"</rp>
                    </ruby>
                </span>
            }
            .into_any()
        },
        None => view! { <span class=class>{text}</span> }.into_any(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn segment_inside_highlight_range_is_highlighted() {
        // The word occupies chars [3, 5): a segment fully inside matches.
        assert!(segment_overlaps_highlight(3, 2, (3, 5)));
    }

    #[test]
    fn segment_straddling_highlight_start_is_highlighted() {
        // Context ruby pairs are not word-aligned: a segment that crosses
        // the range start still overlaps (render must mark it).
        assert!(segment_overlaps_highlight(2, 3, (3, 5)));
    }

    #[test]
    fn segment_straddling_highlight_end_is_highlighted() {
        assert!(segment_overlaps_highlight(4, 3, (3, 5)));
    }

    #[test]
    fn segment_before_highlight_range_is_plain() {
        assert!(!segment_overlaps_highlight(0, 3, (3, 5)));
    }

    #[test]
    fn segment_after_highlight_range_is_plain() {
        assert!(!segment_overlaps_highlight(5, 2, (3, 5)));
    }

    #[test]
    fn absent_highlight_marks_nothing() {
        // Stored offsets [-1,-1] (kana-variant match) — nothing matches,
        // exactly like the renderer contract for unlocatable words.
        assert!(!segment_overlaps_highlight(3, 2, (-1, -1)));
    }

    #[test]
    fn empty_highlight_range_marks_nothing() {
        // Degenerate range (start == end): zero-width, no overlap.
        assert!(!segment_overlaps_highlight(3, 2, (3, 3)));
    }
}
