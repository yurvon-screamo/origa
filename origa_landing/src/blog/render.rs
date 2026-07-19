//! Markdown → sanitized HTML renderer for blog article bodies.
//!
//! Uses `pulldown-cmark` with CommonMark + GFM-tables + strikethrough enabled
//! (matching the options used by the in-app renderer in `origa_ui`), then
//! passes the output through `ammonia` with a policy tuned for article
//! content: external links get `rel="noopener noreferrer"` and a
//! `target="_blank"` (competitor-citation hygiene, no off-site navigation
//! mid-sentence). Default `ammonia` already strips `<script>`, inline event
//! handlers, `javascript:` URLs, and other XSS vectors — we keep that
//! baseline and only widen link handling.

use ammonia::{Builder, UrlRelative};
use pulldown_cmark::{Options, Parser, html};

/// Render a markdown source into a sanitized HTML fragment suitable for
/// `inner_html` injection into a Leptos view. The output is safe against
/// XSS: only an allowlisted set of tags and attributes survives, and
/// external links are stamped with safe `rel`/`target` attributes.
pub fn markdown_to_html(src: &str) -> String {
    let raw_html = render_cmark(src);
    sanitize(&raw_html)
}

fn render_cmark(src: &str) -> String {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_STRIKETHROUGH);

    let parser = Parser::new_ext(src, options);
    let mut output = String::new();
    html::push_html(&mut output, parser);
    output
}

fn sanitize(html: &str) -> String {
    // `link_rel` and `target="_blank"` keep competitor-citation clicks from
    // navigating off the article mid-sentence and from tabnabbing the
    // reader's current tab. `PassThrough` keeps any future relative link
    // intact (the default `Deny` would silently drop them).
    Builder::default()
        .link_rel(Some("noopener noreferrer"))
        .set_tag_attribute_value("a", "target", "_blank")
        .url_relative(UrlRelative::PassThrough)
        .clean(html)
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders_gfm_table() {
        let md = "\
| A | B |
| --- | --- |
| 1 | 2 |
";
        let out = markdown_to_html(md);
        assert!(
            out.contains("<table>"),
            "expected a <table> element; got: {out}"
        );
        assert!(
            out.contains("<th>A</th>"),
            "expected header cell A; got: {out}"
        );
        assert!(
            out.contains("<td>1</td>"),
            "expected data cell 1; got: {out}"
        );
    }

    #[test]
    fn renders_strikethrough() {
        let md = "~~deleted~~";
        let out = markdown_to_html(md);
        assert!(out.contains("<del>deleted</del>"), "got: {out}");
    }

    #[test]
    fn renders_external_link_with_safe_rel_and_target() {
        let md = "[click](https://example.com)";
        let out = markdown_to_html(md);
        assert!(out.contains("href=\"https://example.com\""), "got: {out}");
        assert!(
            out.contains("rel=\"noopener noreferrer\""),
            "external links must carry safe rel; got: {out}"
        );
        assert!(
            out.contains("target=\"_blank\""),
            "external links must open in a new tab; got: {out}"
        );
    }

    #[test]
    fn strips_script_tag() {
        let md = "<script>alert('xss')</script>after";
        let out = markdown_to_html(md);
        assert!(
            !out.contains("<script"),
            "<script> must be stripped; got: {out}"
        );
        assert!(
            !out.contains("alert"),
            "script body must be stripped; got: {out}"
        );
    }

    #[test]
    fn strips_inline_event_handler() {
        let md = "<img src=\"x\" onerror=\"alert('xss')\">";
        let out = markdown_to_html(md);
        assert!(
            !out.contains("onerror"),
            "onerror must be stripped; got: {out}"
        );
    }

    #[test]
    fn strips_javascript_url() {
        let md = "[click](javascript:alert('xss'))";
        let out = markdown_to_html(md);
        assert!(
            !out.contains("javascript:"),
            "javascript: URLs must be stripped; got: {out}"
        );
    }

    #[test]
    fn renders_headings_and_lists_and_emphasis() {
        let md = "# H1\n\n## H2\n\n- one\n- two\n\n**bold** *italic*";
        let out = markdown_to_html(md);
        assert!(out.contains("<h1>H1</h1>"));
        assert!(out.contains("<h2>H2</h2>"));
        assert!(out.contains("<li>one</li>"));
        assert!(out.contains("<li>two</li>"));
        assert!(out.contains("<strong>bold</strong>"));
        assert!(out.contains("<em>italic</em>"));
    }

    #[test]
    fn empty_input_yields_empty_output() {
        assert_eq!(markdown_to_html(""), "");
    }
}
