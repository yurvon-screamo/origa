use leptos::prelude::*;
use origa::dictionary::kanji::KanjiInfo;
use origa::domain::NativeLanguage;
use std::collections::HashSet;

#[component]
pub fn KanjiItem(
    kanji_info: &'static KanjiInfo,
    #[prop(into)] native_language: Signal<NativeLanguage>,
    selected_kanji: RwSignal<HashSet<String>>,
    known_kanji: HashSet<char>,
) -> impl IntoView {
    let kanji_str = kanji_info.kanji().to_string();
    let kanji_str_for_click = kanji_str.clone();
    let kanji_str_for_memo = kanji_str.clone();

    let is_selected = Memo::new(move |_| selected_kanji.get().contains(&kanji_str_for_memo));

    let is_known = known_kanji.contains(&kanji_info.kanji());

    let first_meaning = Memo::new(move |_| {
        let desc = kanji_info.description(&native_language.get());
        truncate_meaning(&desc, 12)
    });

    view! {
        <div
            class=Signal::derive(move || {
                let mut classes = "kanji-grid-tile".to_string();
                if is_selected.get() {
                    classes.push_str(" kanji-grid-tile--selected");
                }
                if is_known {
                    classes.push_str(" kanji-grid-tile--known");
                }
                classes
            })
            data-testid="kanji-drawer-item"
            on:click={
                move |_| {
                    let kanji = kanji_str_for_click.clone();
                    selected_kanji.update(|set| {
                        if set.contains(&kanji) {
                            set.remove(&kanji);
                        } else {
                            set.insert(kanji);
                        }
                    });
                }
            }
        >
            <span class="kanji-grid-tile-char">{kanji_info.kanji()}</span>
            <span class="kanji-grid-tile-meaning">{move || first_meaning.get()}</span>
            <Show when=move || is_selected.get()>
                <span class="kanji-grid-tile-check">"✓"</span>
            </Show>
        </div>
    }
}

fn truncate_meaning(text: &str, max_len: usize) -> String {
    if text.len() <= max_len {
        text.to_string()
    } else {
        let mut end = max_len;
        while !text.is_char_boundary(end) && end > 0 {
            end -= 1;
        }
        format!("{}…", &text[..end])
    }
}
