//! Grammar example fences: the CDN content stores each example as a
//! code fence (JP line, then translation) with inline markdown emphasis
//! inside the body. A plain markdown render keeps fence bodies inside
//! `<pre><code>`, so `**bold**` / `*dialogue*` markers stay raw on
//! screen. [`example_fences_to_paragraphs`] converts example fences
//! into regular paragraphs (line stacking preserved via hard breaks) so
//! the emphasis renders; [`strip_emphasis_markers`] removes the markers
//! for plain-text projections (quiz fronts, TTS sources).

/// Converts example code fences into markdown paragraphs.
///
/// Contract:
/// - the fence info line (the remainder of the opening ` ``` ` line) is
///   dropped — in the content it is empty or a language tag;
/// - blank body lines are dropped, the rest are joined with a hard
///   break (`"  \n"`) so the JP line stays visually stacked over the
///   translation;
/// - `*` passes through (emphasis is the intended content format);
///   literal `<`, `&`, `` ` ``, `_`, `~` and `\` are backslash/HTML
///   escaped so text that was literal inside a code block cannot turn
///   into markup, strikethrough or an entity;
/// - non-fence content passes through verbatim;
/// - an unclosed fence consumes the remainder of the input.
pub fn example_fences_to_paragraphs(markdown: &str) -> String {
    let mut out = String::with_capacity(markdown.len());
    let mut rest = markdown;

    while let Some(open) = rest.find("```") {
        let (before, after_open) = rest.split_at(open);
        push_separator(&mut out, before);
        let body_start = after_open[3..]
            .find('\n')
            .map(|idx| &after_open[3 + idx + 1..])
            .unwrap_or("");
        let (body, tail) = match body_start.find("```") {
            Some(close) => (&body_start[..close], &body_start[close + 3..]),
            None => (body_start, ""),
        };
        let paragraph = fence_body_to_paragraph(body);
        if !paragraph.is_empty() {
            if !out.is_empty() && !out.ends_with("\n\n") {
                out.push_str("\n\n");
            }
            out.push_str(&paragraph);
        }
        rest = tail;
    }
    push_separator(&mut out, rest);
    out
}

/// Appends non-fence content, keeping the paragraph separation that the
/// surrounding markdown already had.
fn push_separator(out: &mut String, text: &str) {
    if text.is_empty() {
        return;
    }
    if !out.is_empty() && !out.ends_with("\n\n") {
        out.push_str("\n\n");
    }
    out.push_str(text.trim_matches('\n'));
    out.push('\n');
}

fn fence_body_to_paragraph(body: &str) -> String {
    let lines: Vec<String> = body
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(escape_literal_markdown)
        .collect();
    lines.join("  \n")
}

/// Backslash/HTML escape for characters that were literal inside a code
/// fence but would become markup in a paragraph. `*` is intentionally
/// NOT escaped — emphasis is the content's intended format.
fn escape_literal_markdown(line: &str) -> String {
    let mut out = String::with_capacity(line.len());
    for ch in line.chars() {
        match ch {
            '\\' => out.push_str("\\\\"),
            '`' => out.push_str("\\`"),
            '<' => out.push_str("&lt;"),
            '&' => out.push_str("&amp;"),
            '_' => out.push_str("\\_"),
            '~' => out.push_str("\\~"),
            _ => out.push(ch),
        }
    }
    out
}

/// Removes `**` / `*` emphasis markers from a single line for
/// plain-text projections (the JP example front, TTS sources). The
/// markers are content format, never literal asterisks (audit of the
/// CDN grammar data: emphasis only).
pub fn strip_emphasis_markers(line: &str) -> String {
    line.replace('*', "")
}

#[cfg(test)]
mod tests {
    use super::*;

    const EXAMPLES: &str = "```\n私は学生です。\nI am a student.\n```\n\n```\n小林さんは先生です。\nKobayashi-san is a teacher.\n```";

    #[test]
    fn fences_become_paragraphs_with_hard_breaks() {
        let out = example_fences_to_paragraphs(EXAMPLES);

        assert!(!out.contains("```"), "fences are gone; got: {out}");
        assert!(
            out.contains("私は学生です。  \nI am a student."),
            "JP line stays stacked over the translation via a hard break; got: {out}"
        );
        assert!(
            out.contains("小林さんは先生です。  \nKobayashi-san is a teacher."),
            "every fence becomes its own paragraph; got: {out}"
        );
    }

    #[test]
    fn emphasis_markers_pass_through_so_they_render() {
        let out = example_fences_to_paragraphs("```\n彼は**[他に強い]**。\nHe is strong.\n```");

        assert!(
            out.contains("彼は**[他に強い]**。"),
            "bracketed bold passes through for the markdown parser; got: {out}"
        );
    }

    #[test]
    fn fence_info_line_is_dropped() {
        let out = example_fences_to_paragraphs("```text\n本文\nBody\n```");

        assert!(
            !out.contains("text"),
            "the info line is not part of the paragraph; got: {out}"
        );
        assert!(out.contains("本文"));
    }

    #[test]
    fn non_fence_content_passes_through_verbatim() {
        let out = example_fences_to_paragraphs("**Заголовок:**\n\n```\n本文\nBody\n```\n\nХвост");

        assert!(
            out.contains("**Заголовок:**"),
            "outside-fence markdown is untouched; got: {out}"
        );
        assert!(out.contains("Хвост"));
    }

    #[test]
    fn blank_body_lines_are_dropped() {
        let out = example_fences_to_paragraphs("```\n\n\n本文\n\nBody\n\n```");

        assert_eq!(out, "本文  \nBody");
    }

    #[test]
    fn empty_fence_body_produces_nothing() {
        let out = example_fences_to_paragraphs("```\n```");

        assert!(out.trim().is_empty(), "got: {out}");
    }

    #[test]
    fn unclosed_fence_consumes_the_rest() {
        let out = example_fences_to_paragraphs("```\n訳があります\n");

        assert!(
            out.contains("訳があります"),
            "unclosed fence body still renders; got: {out}"
        );
        assert!(!out.contains("```"));
    }

    #[test]
    fn literal_markup_characters_are_escaped() {
        let out = example_fences_to_paragraphs("```\na < b & c_d ~ e` f\n```");

        assert!(out.contains("&lt;"), "less-than stays literal; got: {out}");
        assert!(out.contains("&amp;"), "ampersand stays literal; got: {out}");
        assert!(
            out.contains("\\_"),
            "underscore cannot become emphasis; got: {out}"
        );
        assert!(
            out.contains("\\~"),
            "tilde cannot become strikethrough; got: {out}"
        );
        assert!(out.contains("\\`"), "backtick cannot open code; got: {out}");
    }

    #[test]
    fn strippy_removes_emphasis_markers() {
        assert_eq!(
            strip_emphasis_markers("彼は**[他に強い]**。"),
            "彼は[他に強い]。"
        );
        assert_eq!(
            strip_emphasis_markers("*A: どうして泣いているの？*"),
            "A: どうして泣いているの？"
        );
        assert_eq!(strip_emphasis_markers("素朴なテキスト"), "素朴なテキスト");
    }
}
