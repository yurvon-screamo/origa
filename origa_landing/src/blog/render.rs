//! Markdown → sanitized HTML renderer for blog article bodies.
//!
//! Uses `pulldown-cmark` with CommonMark + GFM-tables + strikethrough enabled
//! (matching the options used by the in-app renderer in `origa_ui`), then
//! passes the output through `ammonia` with a policy tuned for article
//! content: external links get `rel="noopener noreferrer"` and a
//! `target="_blank"` (competitor-citation hygiene, no off-site navigation
//! mid-sentence). Default `ammonia` already strips `<script>`, inline event
//! handlers, `javascript:` URLs, and other XSS vectors — we keep that
//! baseline and only widen link handling. Internal (site-relative) links are
//! post-processed to drop the external-link attributes so they navigate in
//! the same tab and stay clean for SEO.

use ammonia::{Builder, UrlRelative};
use pulldown_cmark::{Options, Parser, html};

/// Render a markdown source into a sanitized HTML fragment suitable for
/// `inner_html` injection into a Leptos view. The output is safe against
/// XSS: only an allowlisted set of tags and attributes survives, external
/// links are stamped with safe `rel`/`target` attributes, and internal links
/// are stripped of those attributes for same-tab navigation.
pub fn markdown_to_html(src: &str) -> String {
    let raw_html = render_cmark(src);
    let sanitized = sanitize(&raw_html);
    strip_internal_link_attrs(&sanitized)
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

/// Strip `rel="..."` and `target="_blank"` from anchors whose `href` is
/// site-relative (starts with `/`). External links and absolute URLs keep
/// the safe attributes assigned by [`sanitize`].
///
/// # Why post-process instead of using ammonia per-URL hooks
///
/// Ammonia v4 has no public API for per-URL attribute decisions —
/// `link_rel` and `set_tag_attribute_value` apply uniformly to every `<a>`.
/// The alternative (classification at the `pulldown-cmark` `Event` level,
/// before HTML serialization) would require re-implementing the markdown
/// HTML emitter and is tracked as a future improvement.
///
/// # Contract assumption
///
/// Depends on the exact attribute values and double-quote style emitted by
/// ammonia v4: ` rel="noopener noreferrer"` and ` target="_blank"` as
/// verbatim substrings. The substrings are removed independently of attribute
/// order (ammonia emits `target` before `rel` as of v4; the helper does not
/// rely on that ordering). A minor ammonia release that changes the rel
/// value or quote style would silently break this — the
/// `internal_link_has_no_target_blank` unit test is the only guard. If it
/// fails, the helper needs a corresponding update.
fn strip_internal_link_attrs(html: &str) -> String {
    const ANCHOR_OPEN: &str = "<a ";
    let mut out = String::with_capacity(html.len());
    let mut rest = html;
    while let Some(anchor_idx) = rest.find(ANCHOR_OPEN) {
        out.push_str(&rest[..anchor_idx + ANCHOR_OPEN.len()]);
        rest = &rest[anchor_idx + ANCHOR_OPEN.len()..];
        let close_idx = match rest.find('>') {
            Some(i) => i,
            None => {
                // Unterminated `<a ` — leave the tail untouched.
                out.push_str(rest);
                return out;
            },
        };
        let tag_body = &rest[..close_idx];
        let rewritten = if anchor_href_is_internal(tag_body) {
            tag_body
                .replace(" rel=\"noopener noreferrer\"", "")
                .replace(" target=\"_blank\"", "")
        } else {
            tag_body.to_string()
        };
        out.push_str(&rewritten);
        out.push('>');
        rest = &rest[close_idx + 1..];
    }
    out.push_str(rest);
    out
}

/// True iff the `href="..."` attribute on an anchor tag body points at a
/// site-relative URL (starts with `/`). External URLs, protocol-relative
/// URLs (`//host`), and malformed anchors all return false.
fn anchor_href_is_internal(tag_body: &str) -> bool {
    let Some(href_idx) = tag_body.find("href=\"") else {
        return false;
    };
    let value_start = href_idx + "href=\"".len();
    let Some(end_offset) = tag_body[value_start..].find('"') else {
        return false;
    };
    let href = &tag_body[value_start..value_start + end_offset];
    href.starts_with('/')
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
    fn internal_link_has_no_target_blank() {
        // Same-tab navigation for site-relative URLs is the UX and SEO
        // contract — the post-processor in `markdown_to_html` must strip
        // the external-link attributes from internal anchors.
        let md = "[see the comparison](/compare)";
        let out = markdown_to_html(md);
        assert!(
            out.contains(r#"href="/compare""#),
            "internal link must keep its href verbatim; got: {out}"
        );
        assert!(
            !out.contains("target=\"_blank\""),
            "internal links must not carry target=_blank; got: {out}"
        );
        assert!(
            !out.contains("rel=\"noopener noreferrer\""),
            "internal links must not carry external rel; got: {out}"
        );
    }

    #[test]
    fn internal_link_with_locale_prefix_stays_internal() {
        let md = "[полное сравнение](/ru/compare)";
        let out = markdown_to_html(md);
        assert!(
            out.contains(r#"href="/ru/compare""#),
            "locale-prefixed internal link must keep its href; got: {out}"
        );
        assert!(
            !out.contains("target=\"_blank\""),
            "locale-prefixed internal links are still internal; got: {out}"
        );
    }

    #[test]
    fn external_and_internal_links_coexist() {
        let md = "[external](https://example.com) and [internal](/features)";
        let out = markdown_to_html(md);
        // Ammonia v4 emits `target` before `rel`; assert each attribute
        // independently so the test does not lock in a specific order.
        let external_start = out
            .find(r#"href="https://example.com""#)
            .expect("external href must be present");
        let external_end = out[external_start..].find('>').unwrap() + external_start;
        let external_tag = &out[external_start..=external_end];
        assert!(
            external_tag.contains("rel=\"noopener noreferrer\""),
            "external link must keep safe rel; got: {external_tag}"
        );
        assert!(
            external_tag.contains("target=\"_blank\""),
            "external link must keep target=_blank; got: {external_tag}"
        );
        assert!(
            out.contains(r#"href="/features""#),
            "internal link must keep its href; got: {out}"
        );
        let internal_start = out.find(r#"href="/features""#).unwrap();
        let internal_end = out[internal_start..].find('>').unwrap() + internal_start;
        let internal_tag = &out[internal_start..=internal_end];
        assert!(
            !internal_tag.contains("target=\"_blank\""),
            "internal anchor tag must not carry target=_blank; got: {internal_tag}"
        );
        assert!(
            !internal_tag.contains("rel=\"noopener noreferrer\""),
            "internal anchor tag must not carry external rel; got: {internal_tag}"
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
