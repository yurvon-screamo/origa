use ammonia::clean;
use leptos::prelude::*;
use pulldown_cmark::{Options, Parser, html};

#[derive(Clone, Copy, PartialEq, Default, Debug)]
pub enum MarkdownVariant {
    #[default]
    Default,
    Compact,
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

#[component]
pub fn MarkdownText(
    #[prop(into)] content: Signal<String>,
    #[prop(optional, into)] variant: Signal<MarkdownVariant>,
    #[prop(optional, into)] class: Signal<String>,
) -> impl IntoView {
    let html_content = Memo::new(move |_| render_markdown(&content.get()));

    view! {
        <div class=move || {
            let variant_class = match variant.get() {
                MarkdownVariant::Default => "prose prose-sm",
                MarkdownVariant::Compact => "prose prose-xs",
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
}
