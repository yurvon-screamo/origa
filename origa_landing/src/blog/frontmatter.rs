//! Hand-rolled YAML-subset parser for article frontmatter.
//!
//! The blog frontmatter is a fixed-shape record of eight scalar/array fields
//! delimited by `---` lines. Pulling in a YAML crate for that is
//! over-engineering — this parser handles exactly the fields we use, fails
//! loudly on anything malformed, and stays inside one screen per function.

use std::str::FromStr;

use crate::content::Locale;

/// Publication status declared in the article's frontmatter. The registry
/// refuses to ship a `Draft` article: it panics on first access so a draft
/// can never reach production. This is a programmer error, not a runtime
/// condition — drafts belong on a branch, not behind a flag.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArticleStatus {
    Draft,
    Ready,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Frontmatter {
    pub title: String,
    pub locale: Locale,
    pub meta_title: String,
    pub meta_description: String,
    pub target_keywords: Vec<String>,
    /// ISO-8601 date (`YYYY-MM-DD`) lifted verbatim from the `lastmod` field.
    /// Used for Schema.org `dateModified` (and for `datePublished` when no
    /// `published` field is present in the frontmatter).
    pub lastmod: String,
    /// Optional ISO-8601 date (`YYYY-MM-DD`) of first publication. When
    /// present, sourced as Schema.org `datePublished`; when absent,
    /// `datePublished` falls back to `lastmod`.
    pub published: Option<String>,
    pub status: ArticleStatus,
}

#[derive(Debug, thiserror::Error)]
pub enum FrontmatterError {
    #[error("missing opening `---` delimiter")]
    MissingOpeningDelimiter,
    #[error("missing closing `---` delimiter")]
    MissingClosingDelimiter,
    #[error("missing required field: {0}")]
    MissingField(&'static str),
    #[error("duplicate key `{0}` (only one value per key is allowed)")]
    DuplicateKey(String),
    #[error("invalid locale `{0}` (expected en/ru/ko/vi)")]
    InvalidLocale(String),
    #[error("invalid status `{0}` (expected draft or ready)")]
    InvalidStatus(String),
    #[error("invalid target_keywords value: {0}")]
    InvalidKeywords(String),
    #[error("invalid lastmod `{0}` (expected YYYY-MM-DD)")]
    InvalidLastmod(String),
    #[error("invalid published `{0}` (expected YYYY-MM-DD)")]
    InvalidPublished(String),
    #[error(
        "published `{published}` is after lastmod `{lastmod}` (an article cannot be modified before it was first published)"
    )]
    PublishedAfterLastmod { published: String, lastmod: String },
    #[error("unknown frontmatter key `{0}` (typo or unsupported field)")]
    UnknownKey(String),
}

/// Split a markdown source into `(frontmatter, body)`. The frontmatter is the
/// YAML-like block between the first two `---` lines; the body is everything
/// after the closing `---`. A leading byte-order-mark or whitespace before
/// the opening `---` is not tolerated — every article file must start with
/// `---\n` as its first line.
pub fn split_frontmatter(src: &str) -> Result<(&str, &str), FrontmatterError> {
    let after_open = src
        .strip_prefix("---\n")
        .ok_or(FrontmatterError::MissingOpeningDelimiter)?;

    let close_idx = after_open
        .find("\n---\n")
        .ok_or(FrontmatterError::MissingClosingDelimiter)?;
    let yaml_block = &after_open[..close_idx];
    // Skip past the closing `\n---\n` so the body starts on its own line.
    let body = &after_open[close_idx + "\n---\n".len()..];
    Ok((yaml_block, body))
}

/// Parse a frontmatter YAML block into a [`Frontmatter`]. The block is the
/// text between the `---` delimiters, exclusive. Fails on duplicate keys,
/// unknown keys (catches typos like `mete_title` for `meta_title`), and
/// `lastmod` values that are not `YYYY-MM-DD`.
pub fn parse(yaml: &str) -> Result<Frontmatter, FrontmatterError> {
    let mut title = None;
    let mut locale = None;
    let mut meta_title = None;
    let mut meta_description = None;
    let mut target_keywords = None;
    let mut lastmod = None;
    let mut published = None;
    let mut status = None;

    let mut seen: std::collections::HashSet<&str> = std::collections::HashSet::new();

    for line in yaml.lines() {
        let Some((key, value)) = split_kv(line) else {
            continue;
        };
        if !seen.insert(key) {
            return Err(FrontmatterError::DuplicateKey(key.to_string()));
        }
        match key {
            "title" => title = Some(unquote(value)),
            "locale" => locale = Some(parse_locale(value)?),
            "meta_title" => meta_title = Some(unquote(value)),
            "meta_description" => meta_description = Some(unquote(value)),
            "target_keywords" => target_keywords = Some(parse_keywords(value)?),
            "lastmod" => {
                let v = unquote(value);
                if !is_iso_date(&v) {
                    return Err(FrontmatterError::InvalidLastmod(v));
                }
                lastmod = Some(v);
            },
            "published" => {
                let v = unquote(value);
                if !is_iso_date(&v) {
                    return Err(FrontmatterError::InvalidPublished(v));
                }
                published = Some(v);
            },
            "status" => status = Some(parse_status(value)?),
            // `slug` is documented in the source file but not consumed at
            // runtime — the canonical slug is the filename, so a mismatched
            // frontmatter slug cannot break routing. It is the only key
            // accepted-but-ignored; any other unknown key is a likely typo
            // and is rejected so future field renames surface immediately.
            "slug" => {},
            other => return Err(FrontmatterError::UnknownKey(other.to_string())),
        }
    }

    Ok({
        let fm = Frontmatter {
            title: title.ok_or(FrontmatterError::MissingField("title"))?,
            locale: locale.ok_or(FrontmatterError::MissingField("locale"))?,
            meta_title: meta_title.ok_or(FrontmatterError::MissingField("meta_title"))?,
            meta_description: meta_description
                .ok_or(FrontmatterError::MissingField("meta_description"))?,
            target_keywords: target_keywords
                .ok_or(FrontmatterError::MissingField("target_keywords"))?,
            lastmod: lastmod.ok_or(FrontmatterError::MissingField("lastmod"))?,
            published,
            status: status.ok_or(FrontmatterError::MissingField("status"))?,
        };
        // Cross-field invariant: `published` (first publication date) cannot
        // be later than `lastmod` (last edit date). Both are ISO-8601
        // `YYYY-MM-DD`, so lexicographic comparison is also chronological.
        // Without this check, JSON-LD `dateModified` < `datePublished` would
        // be emitted, which Google flags as inconsistent in the Article entity.
        if let Some(ref p) = fm.published {
            if p > &fm.lastmod {
                return Err(FrontmatterError::PublishedAfterLastmod {
                    published: p.clone(),
                    lastmod: fm.lastmod.clone(),
                });
            }
        }
        fm
    })
}

/// Validate `YYYY-MM-DD` without pulling in a regex dependency. Matches the
/// sitemap validator in `tests/sitemap.rs` — the frontmatter `lastmod` is the
/// value that ends up in JSON-LD `dateModified`; `datePublished` falls back to
/// `lastmod` when `published` is unset, so both must be real ISO-8601 dates or
/// Google flags the Article entity.
fn is_iso_date(value: &str) -> bool {
    let bytes = value.as_bytes();
    bytes.len() == 10
        && bytes[4] == b'-'
        && bytes[7] == b'-'
        && bytes[0..4].iter().all(|b| b.is_ascii_digit())
        && bytes[5..7].iter().all(|b| b.is_ascii_digit())
        && bytes[8..10].iter().all(|b| b.is_ascii_digit())
}

fn split_kv(line: &str) -> Option<(&str, &str)> {
    let line = line.trim();
    if line.is_empty() || line.starts_with('#') {
        return None;
    }
    let colon = line.find(':')?;
    let key = line[..colon].trim();
    let value = line[colon + 1..].trim();
    if key.is_empty() {
        return None;
    }
    Some((key, value))
}

fn unquote(value: &str) -> String {
    let value = value.trim();
    if value.len() >= 2 && value.starts_with('"') && value.ends_with('"') {
        value[1..value.len() - 1].to_string()
    } else {
        value.to_string()
    }
}

fn parse_locale(value: &str) -> Result<Locale, FrontmatterError> {
    let value = unquote(value);
    Locale::from_str(&value).map_err(|()| FrontmatterError::InvalidLocale(value))
}

fn parse_status(value: &str) -> Result<ArticleStatus, FrontmatterError> {
    match unquote(value).as_str() {
        "draft" => Ok(ArticleStatus::Draft),
        "ready" => Ok(ArticleStatus::Ready),
        other => Err(FrontmatterError::InvalidStatus(other.to_string())),
    }
}

/// Parse a YAML inline-array of strings: `["a", "b", "c"]`. Quoted and
/// unquoted entries are both accepted; surrounding whitespace is trimmed.
fn parse_keywords(value: &str) -> Result<Vec<String>, FrontmatterError> {
    let value = value.trim();
    let inner = value
        .strip_prefix('[')
        .and_then(|v| v.strip_suffix(']'))
        .ok_or_else(|| FrontmatterError::InvalidKeywords(value.to_string()))?;
    if inner.trim().is_empty() {
        return Ok(Vec::new());
    }
    inner
        .split(',')
        .map(|entry| Ok(unquote(entry.trim())))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = "\
title: \"Anki Alternative for Japanese\"
slug: /blog/anki-alternative-japanese
locale: en
meta_title: \"Anki Alternative (2026)\"
meta_description: \"Field guide.\"
target_keywords: [\"anki alternative japanese\", \"japanese flashcards app\"]
lastmod: 2026-07-18
status: ready
";

    #[test]
    fn parse_extracts_all_fields() {
        let fm = parse(SAMPLE).expect("well-formed frontmatter must parse");
        assert_eq!(fm.title, "Anki Alternative for Japanese");
        assert_eq!(fm.locale, Locale::En);
        assert_eq!(fm.meta_title, "Anki Alternative (2026)");
        assert_eq!(fm.meta_description, "Field guide.");
        assert_eq!(
            fm.target_keywords,
            vec!["anki alternative japanese", "japanese flashcards app"]
        );
        assert_eq!(fm.lastmod, "2026-07-18");
        assert_eq!(fm.status, ArticleStatus::Ready);
    }

    #[test]
    fn parse_accepts_draft_status() {
        let src = SAMPLE.replacen("ready", "draft", 1);
        let fm = parse(&src).expect("draft is a valid status value");
        assert_eq!(fm.status, ArticleStatus::Draft);
    }

    #[test]
    fn parse_rejects_unknown_locale() {
        let src = SAMPLE.replacen("locale: en", "locale: zh", 1);
        let err = parse(&src).expect_err("zh is not a supported locale");
        assert!(matches!(err, FrontmatterError::InvalidLocale(_)));
    }

    #[test]
    fn parse_rejects_unknown_status() {
        let src = SAMPLE.replacen("status: ready", "status: published", 1);
        let err = parse(&src).expect_err("published is not a valid status");
        assert!(matches!(err, FrontmatterError::InvalidStatus(_)));
    }

    #[test]
    fn parse_rejects_malformed_keywords() {
        let src = SAMPLE.replacen("target_keywords:", "target_keywords: not-an-array", 1);
        let err = parse(&src).expect_err("keywords must be a YAML array");
        assert!(matches!(err, FrontmatterError::InvalidKeywords(_)));
    }

    #[test]
    fn parse_rejects_missing_required_field() {
        let src = SAMPLE.replacen("lastmod: 2026-07-18\n", "", 1);
        let err = parse(&src).expect_err("lastmod is required");
        assert!(matches!(err, FrontmatterError::MissingField("lastmod")));
    }

    #[test]
    fn split_frontmatter_extracts_yaml_block_and_body() {
        let src = "---\ntitle: x\nlastmod: 2026-01-01\nlocale: en\nmeta_title: x\nmeta_description: x\ntarget_keywords: []\nstatus: ready\n---\n# Heading\n";
        let (yaml, body) = split_frontmatter(src).expect("must split");
        assert!(yaml.contains("title: x"));
        assert!(body.starts_with("# Heading"));
    }

    #[test]
    fn split_frontmatter_rejects_missing_open_delimiter() {
        let src = "title: x\n---\nbody";
        let err = split_frontmatter(src).expect_err("opening `---` is required");
        assert!(matches!(err, FrontmatterError::MissingOpeningDelimiter));
    }

    #[test]
    fn split_frontmatter_rejects_missing_close_delimiter() {
        let src = "---\ntitle: x\nbody without close";
        let err = split_frontmatter(src).expect_err("closing `---` is required");
        assert!(matches!(err, FrontmatterError::MissingClosingDelimiter));
    }

    #[test]
    fn parse_rejects_duplicate_key() {
        // A duplicate key in YAML is undefined behaviour; reject it
        // explicitly so an editor cannot silently override an earlier value.
        let src = SAMPLE.to_string() + "\nlastmod: 2026-12-31";
        let err = parse(&src).expect_err("duplicate lastmod must be rejected");
        assert!(matches!(err, FrontmatterError::DuplicateKey(k) if k == "lastmod"));
    }

    #[test]
    fn parse_rejects_unknown_key() {
        // A typo like `mete_title` (instead of `meta_title`) currently
        // produces a confusing `missing required field: meta_title` later.
        // Surface the typo directly so the author can fix the source.
        let src = SAMPLE.replacen("meta_title:", "mete_title:", 1);
        let err = parse(&src).expect_err("unknown key must be rejected");
        assert!(matches!(err, FrontmatterError::UnknownKey(k) if k == "mete_title"));
    }

    #[test]
    fn parse_rejects_non_iso_lastmod() {
        let src = SAMPLE.replacen("lastmod: 2026-07-18", "lastmod: July 18, 2026", 1);
        let err = parse(&src).expect_err("lastmod must be YYYY-MM-DD");
        assert!(matches!(err, FrontmatterError::InvalidLastmod(_)));
    }

    #[test]
    fn parse_published_defaults_to_none_when_absent() {
        let fm = parse(SAMPLE).expect("SAMPLE without `published` must parse");
        assert!(
            fm.published.is_none(),
            "published must default to None when the field is absent"
        );
    }

    #[test]
    fn parse_extracts_published_when_present() {
        // lastmod in SAMPLE is 2026-07-18; published must be ≤ lastmod
        // (cross-field invariant enforced by `parse_rejects_published_after_lastmod`).
        let src = SAMPLE.to_string() + "\npublished: 2026-07-15";
        let fm = parse(&src).expect("valid `published` value must parse");
        assert_eq!(fm.published.as_deref(), Some("2026-07-15"));
    }

    #[test]
    fn parse_rejects_non_iso_published() {
        let src = SAMPLE.to_string() + "\npublished: July 19, 2026";
        let err = parse(&src).expect_err("published must be YYYY-MM-DD");
        assert!(
            matches!(err, FrontmatterError::InvalidPublished(_)),
            "non-ISO published must surface its own error variant, got {err:?}"
        );
    }

    #[test]
    fn parse_rejects_published_after_lastmod() {
        // Cross-field invariant: an article cannot be modified before it was
        // first published. lastmod in SAMPLE is 2026-07-18; published after
        // it must surface a dedicated error variant.
        let src = SAMPLE.to_string() + "\npublished: 2026-12-31";
        let err = parse(&src).expect_err("published > lastmod must be rejected");
        assert!(
            matches!(err, FrontmatterError::PublishedAfterLastmod { .. }),
            "published-after-lastmod must surface PublishedAfterLastmod, got {err:?}"
        );
    }

    #[test]
    fn parse_accepts_published_equal_to_lastmod() {
        // Same-day publish-and-edit is allowed: published == lastmod is the
        // edge of the invariant (lex compare is `>` not `>=`).
        let src = SAMPLE.to_string() + "\npublished: 2026-07-18";
        let fm = parse(&src).expect("published == lastmod must parse");
        assert_eq!(fm.published.as_deref(), Some("2026-07-18"));
    }

    #[test]
    fn parse_accepts_documented_slug_key() {
        // `slug` is the only accepted-but-unused key: it documents the
        // intended URL path in the source file without affecting routing.
        // Removing this exception would break both existing articles.
        let src = SAMPLE.to_string();
        let fm = parse(&src).expect("slug key must not be treated as unknown");
        assert_eq!(fm.title, "Anki Alternative for Japanese");
    }
}
