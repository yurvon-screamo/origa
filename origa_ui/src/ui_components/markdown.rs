use std::collections::HashSet;

use ammonia::clean;
use ego_tree::NodeRef;
use leptos::prelude::*;
use origa::domain::furiganize_text;
use pulldown_cmark::{html, Options, Parser};
use scraper::{Html, Node};

#[derive(Clone, Copy, PartialEq, Default, Debug)]
pub enum MarkdownVariant {
    #[default]
    Default,
    Compact,
    Large,
}

fn render_markdown(content: &str) -> String {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TABLES);

    let parser = Parser::new_ext(content, options);
    let mut html_output = String::new();
    html::push_html(&mut html_output, parser);

    clean(&html_output)
}

const SKIP_TAGS: &[&str] = &["pre", "ruby", "rt", "rp"];

fn add_furigana_to_html(html: &str, known_kanji: &HashSet<String>) -> String {
    let document = Html::parse_document(html);
    let mut result = String::new();

    fn process_node(
        node_ref: NodeRef<'_, Node>,
        output: &mut String,
        in_skip: bool,
        known_kanji: &HashSet<String>,
    ) {
        match node_ref.value() {
            Node::Text(text) => {
                let text_str: &str = text;
                if in_skip {
                    output.push_str(text_str);
                } else {
                    match furiganize_text(text_str, known_kanji) {
                        Ok(furigana) => output.push_str(&furigana),
                        Err(_) => output.push_str(text_str),
                    }
                }
            }
            Node::Element(elem) => {
                let tag = elem.name();
                let should_skip = in_skip || SKIP_TAGS.contains(&tag);

                output.push_str(&format!("<{}", tag));
                for (name, value) in elem.attrs() {
                    output.push_str(&format!(" {}=\"{}\"", name, value));
                }
                output.push('>');

                for child in node_ref.children() {
                    process_node(child, output, should_skip, known_kanji);
                }

                output.push_str(&format!("</{}>", tag));
            }
            _ => {
                for child in node_ref.children() {
                    process_node(child, output, in_skip, known_kanji);
                }
            }
        }
    }

    for node_ref in document.tree.root().children() {
        process_node(node_ref, &mut result, false, known_kanji);
    }

    result
}

#[component]
pub fn MarkdownText(
    #[prop(into)] content: Signal<String>,
    known_kanji: HashSet<String>,
    #[prop(optional, into)] variant: Signal<MarkdownVariant>,
    #[prop(optional, into)] class: Signal<String>,
    #[prop(optional, default = true)] furigana: bool,
) -> impl IntoView {
    let html_content = Memo::new(move |_| {
        let rendered = render_markdown(&content.get());
        if furigana {
            add_furigana_to_html(&rendered, &known_kanji)
        } else {
            rendered
        }
    });

    view! {
        <div class=move || {
            let variant_class = match variant.get() {
                MarkdownVariant::Default => "prose prose-sm",
                MarkdownVariant::Compact => "prose prose-xs",
                MarkdownVariant::Large => "prose prose-lg",
            };
            format!("markdown-text {} {}", variant_class, class.get())
        }>
            <div inner_html=move || html_content.get() />
        </div>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_basic_markdown() {
        let input = "**bold** and *italic*";
        let output = render_markdown(input);
        assert!(output.contains("<strong>bold</strong>"));
        assert!(output.contains("<em>italic</em>"));
    }

    #[test]
    fn test_render_heading() {
        let input = "# Heading 1\n## Heading 2";
        let output = render_markdown(input);
        assert!(output.contains("<h1>Heading 1</h1>"));
        assert!(output.contains("<h2>Heading 2</h2>"));
    }

    #[test]
    fn test_render_list() {
        let input = "- item 1\n- item 2";
        let output = render_markdown(input);
        assert!(output.contains("<ul>"));
        assert!(output.contains("<li>item 1</li>"));
        assert!(output.contains("<li>item 2</li>"));
    }

    #[test]
    fn test_render_code() {
        let input = "`inline code`";
        let output = render_markdown(input);
        assert!(output.contains("<code>inline code</code>"));
    }

    #[test]
    fn test_render_link() {
        let input = "[text](https://example.com)";
        let output = render_markdown(input);
        assert!(output.contains("<a"));
        assert!(output.contains("href=\"https://example.com\""));
        assert!(output.contains(">text</a>"));
    }

    #[test]
    fn test_sanitize_script_tag() {
        let input = "<script>alert('xss')</script>";
        let output = render_markdown(input);
        assert!(!output.contains("<script>"));
        assert!(!output.contains("alert"));
    }

    #[test]
    fn test_sanitize_javascript_link() {
        let input = "[click](javascript:alert('xss'))";
        let output = render_markdown(input);
        assert!(!output.contains("javascript:"));
    }

    #[test]
    fn test_sanitize_event_handler() {
        let input = "<img src=\"x\" onerror=\"alert('xss')\">";
        let output = render_markdown(input);
        assert!(!output.contains("onerror"));
    }

    #[test]
    fn test_empty_input() {
        let output = render_markdown("");
        assert!(output.is_empty());
    }

    #[test]
    fn test_add_furigana_preserves_html_structure() {
        let html = "<p>Hello world</p>";
        let known_kanji = HashSet::new();
        let output = add_furigana_to_html(html, &known_kanji);
        assert!(output.contains("<p>"));
        assert!(output.contains("</p>"));
        assert!(output.contains("Hello"));
        assert!(output.contains("world"));
    }

    #[test]
    fn test_add_furigana_skips_code_tag() {
        let html = "<code>test</code>";
        let known_kanji = HashSet::new();
        let output = add_furigana_to_html(html, &known_kanji);
        assert!(output.contains("<code>test</code>"));
    }

    #[test]
    fn test_add_furigana_skips_pre_tag() {
        let html = "<pre>test</pre>";
        let known_kanji = HashSet::new();
        let output = add_furigana_to_html(html, &known_kanji);
        assert!(output.contains("<pre>test</pre>"));
    }

    #[test]
    fn test_add_furigana_skips_ruby_tag() {
        let html = "<ruby>食<rt>しょく</rt></ruby>";
        let known_kanji = HashSet::new();
        let output = add_furigana_to_html(html, &known_kanji);
        assert!(output.contains("<ruby>"));
        assert!(output.contains("<rt>"));
    }

    #[test]
    fn test_add_furigana_preserves_links() {
        let html = "<a href=\"https://example.com\">link</a>";
        let known_kanji = HashSet::new();
        let output = add_furigana_to_html(html, &known_kanji);
        assert!(output.contains("href=\"https://example.com\""));
        assert!(output.contains(">link</a>"));
    }

    #[test]
    fn test_add_furigana_nested_elements() {
        let html = "<div><p>text</p></div>";
        let known_kanji = HashSet::new();
        let output = add_furigana_to_html(html, &known_kanji);
        assert!(output.contains("<div>"));
        assert!(output.contains("<p>"));
        assert!(output.contains("text"));
        assert!(output.contains("</p>"));
        assert!(output.contains("</div>"));
    }

    #[test]
    fn test_add_furigana_code_inside_p() {
        let html = "<p>text <code>code</code> more</p>";
        let known_kanji = HashSet::new();
        let output = add_furigana_to_html(html, &known_kanji);
        assert!(output.contains("text"));
        assert!(output.contains("<code>code</code>"));
        assert!(output.contains("more"));
    }
}
