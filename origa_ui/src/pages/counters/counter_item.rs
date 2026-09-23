use leptos::prelude::*;
use origa::dictionary::counters::CounterEntry;
use origa::domain::NativeLanguage;
use std::collections::HashSet;

#[component]
pub fn CounterItem(
    counter_entry: &'static CounterEntry,
    #[prop(into)] native_language: Signal<NativeLanguage>,
    selected_counters: RwSignal<HashSet<String>>,
    known_counters: HashSet<String>,
) -> impl IntoView {
    let suffix = counter_entry.suffix().to_string();
    let suffix_for_click = suffix.clone();
    let suffix_for_memo = suffix.clone();

    let is_selected = Memo::new(move |_| selected_counters.get().contains(&suffix_for_memo));

    let is_known = known_counters.contains(counter_entry.suffix());

    let first_meaning = Memo::new(move |_| {
        let gloss = origa::dictionary::counters::gloss_for(counter_entry, native_language.get());
        truncate_meaning(gloss, 12)
    });

    view! {
        <div
            class=Signal::derive(move || {
                let mut classes = "counter-grid-tile".to_string();
                if is_selected.get() {
                    classes.push_str(" counter-grid-tile--selected");
                }
                if is_known {
                    classes.push_str(" counter-grid-tile--known");
                }
                classes
            })
            data-testid="counters-drawer-item"
            on:click={
                move |_| {
                    let suffix = suffix_for_click.clone();
                    selected_counters.update(|set| {
                        if set.contains(&suffix) {
                            set.remove(&suffix);
                        } else {
                            set.insert(suffix);
                        }
                    });
                }
            }
        >
            <span class="counter-grid-tile-char">{counter_entry.suffix()}</span>
            <span class="counter-grid-tile-meaning">{move || first_meaning.get()}</span>
            <Show when=move || is_selected.get()>
                <span class="counter-grid-tile-check">"✓"</span>
            </Show>
        </div>
    }
}

/// Однострочное превью глоссы — зеркало `truncate_meaning` кандзи-плитки.
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
