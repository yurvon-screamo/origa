# ADR-032: Blog Internal-Link Sanitize Policy Differentiation

## Status

Accepted

## Date

2026-07-21

## Context

The blog renderer (`origa_landing/src/blog/render.rs`) sanitizes markdown-
generated HTML through `ammonia` v4 with a blanket link policy:

```rust
Builder::default()
    .link_rel(Some("noopener noreferrer"))
    .set_tag_attribute_value("a", "target", "_blank")
    .url_relative(UrlRelative::PassThrough)
```

This was correct when every link in every article was an external competitor
citation (WaniKani, Bunpro, Migii, Anki docs). `target="_blank"` keeps the
reader on the article while peeking at the competitor; `rel="noopener
noreferrer"` blocks tabnabbing and referrer leakage to third parties.

When the blog-hub polish work (PR #277) added internal links — to `/compare`,
`/features`, `/download`, with locale prefixes (`/ru/compare`, `/ko/features`,
`/vi/download`) — the blanket policy produced a UX and SEO regression:

- **UX:** every internal link opened a new tab, fragmenting the reading
  session instead of navigating within it.
- **SEO:** internal links carrying `rel="noreferrer"` is not strictly harmful
  but is unusual; the substantive issue is `target="_blank"`, which signals
  "off-site" to crawlers analyzing internal anchor topology.

Ammonia v4 has no public API for per-URL attribute decisions. `link_rel`
takes `Option<&str>` applied to every `<a>`; `set_tag_attribute_value` is
likewise tag-wide. Differentiating internal from external requires either
post-processing ammonia's output or re-implementing the markdown HTML
emitter on top of `pulldown-cmark` `Event`s.

## Decision

Post-process ammonia's output with a string-based helper
(`strip_internal_link_attrs` in `render.rs`) that removes the verbatim
substrings `rel="noopener noreferrer"` and `target="_blank"` from `<a>`
tags whose `href` starts with `/` (site-relative). External and absolute
URLs keep the safe attributes unchanged.

The helper operates on the post-sanitize HTML string. It does not introduce
new dependencies (the workspace already lists `scraper`, but the landing
crate does not depend on it; pulling it in for this one use case was
rejected as disproportionate).

## Consequences

**Positive:**

- Internal links navigate in the same tab — readers stay in the article
  flow when clicking to `/compare`, `/features`, `/download`.
- No new dependencies; the helper is ~30 lines of straightforward byte-
  scanning code.
- The change is backward-compatible with existing external-link tests
  (`ru_article_has_inline_competitor_citation`) and the new
  `external_links_in_article_keep_safe_attrs` integration test pins the
  contract.

**Negative:**

- The helper depends on the exact attribute values and double-quote style
  emitted by ammonia v4: `rel="noopener noreferrer"` and `target="_blank"`
  as verbatim substrings. Ammonia's output format (attribute values, quote
  style, attribute ordering) is **not** part of its public API contract. A
  minor ammonia release that changes the rel value, quote style, or attribute
  ordering would silently break the stripping. The `internal_link_has_no_target_blank`
  unit test is the only guard.
- A more robust alternative — classifying links at the `pulldown-cmark`
  `Event::Start(Tag::Link { dest_url, .. })` level before HTML serialization —
  was rejected for this slice because it requires re-implementing the
  markdown HTML emitter (`html::push_html`). It remains a candidate if
  ammonia output drift becomes a recurring maintenance burden.

## Verification

- `render::tests::internal_link_has_no_target_blank` — pins the contract on
  a synthetic `[x](/compare)` input.
- `render::tests::external_and_internal_links_coexist` — both classes of
  link survive in the same render with the correct attributes.
- `tests/blog.rs::internal_links_in_article_do_not_open_new_tab` — pins the
  contract on a real rendered article (`/blog/anki-alternative-japanese`).
- `tests/blog.rs::external_links_in_article_keep_safe_attrs` — pins the
  symmetric contract: the WaniKani competitor citation keeps both
  `rel="noopener noreferrer"` and `target="_blank"`.

If a future ammonia bump fails `internal_link_has_no_target_blank`, the fix
is local: update the verbatim substrings in `strip_internal_link_attrs` (or
migrate to the `pulldown-cmark` Event-level approach outlined above).
