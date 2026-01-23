use dioxus::prelude::*;

use super::FuriganaText;

fn render_formatted_lines(text: &str, text_class: &str, show_furigana: bool) -> Vec<Element> {
    let lines: Vec<&str> = text.split('\n').collect();
    let mut elements = Vec::new();

    for line in lines.iter() {
        let trimmed = line.trim();
        if trimmed.starts_with("- ") {
            let content = trimmed.strip_prefix("- ").unwrap_or(trimmed).to_string();
            elements.push(rsx! {
                div { class: "{text_class}",
                    span { class: "mr-2", "•" }
                    span { "{content}" }
                }
            });
        } else if trimmed.starts_with("> ") {
            let content = trimmed.strip_prefix("> ").unwrap_or(trimmed).to_string();
            elements.push(rsx! {
                div { class: "{text_class} text-slate-600 italic",
                    span { class: "mr-1", ">" }
                    span { "{content}" }
                }
            });
        } else if !trimmed.is_empty() {
            let content = trimmed.to_string();
            elements.push(rsx! {
                div { class: "{text_class}",
                    FuriganaText { text: content, show_furigana, class: None }
                }
            });
        }
    }

    elements
}

#[component]
pub fn FormattedTranslation(text: String, class: Option<String>, show_furigana: bool) -> Element {
    let text_class = class.unwrap_or_else(|| "text-lg md:text-xl".to_string());

    rsx! {
        div { class: "space-y-1",
            for element in render_formatted_lines(&text, &text_class, show_furigana) {
                {element}
            }
        }
    }
}
