# ADR-017: AI Crawler Stance (Allow, Disable Cloudflare AI Audit)

## Status

Accepted

## Date

2026-06-26

## Context

Search is shifting from link-based engines (Google, Yandex, Bing) to
generative answer engines (ChatGPT, Perplexity, Claude, Google AI Overviews,
Copilot). For a product like Origa — whose target queries ("japanese learning
app offline", "anki alternative", per-language variants) are exactly the kind
users now put to assistants — being absent from the generative index is a
growing acquisition risk. This is commonly called Generative Engine
Optimization (GEO).

AI crawlers identify themselves via distinct user-agents (`GPTBot`,
`ClaudeBot`, `PerplexityBot`, `Google-Extended`, `CCBot`, `anthropic-ai`,
etc.). A site controls them through `robots.txt`. There are two opposing
strategies:

1. **Allow all** — maximize the chance the product is cited in answers.
2. **Block** — prevent training/scraping (some publishers block to protect
   paywalled or copyrighted content, or to bargain via "Pay Per Crawl").

Origa's content is a marketing surface (not paywalled journalism) and the
product benefits from being cited, so the value of being in the generative
index outweighs the cost of being trained on public marketing copy.

A second, operational concern: Cloudflare's "AI Audit" / "Pay Per Crawl"
feature, when enabled on the `uwuwu.net` zone, **injects its own managed
`robots.txt` block** ahead of the user-managed file — including
`Disallow: /` for several AI user-agents and a `Content-Signal` header. This
silently overrides whatever `robots.txt` we ship. ADR-011 documents this as
RC-2 and provides a dashboard runbook to disable it.

## Decision

### Allow all AI crawlers via `robots.txt`

`public/robots.txt` keeps the existing `User-agent: * / Allow: /` (which is
already an allow-all) and gains a comment block documenting the stance:

```
# AI crawlers (GPTBot, ClaudeBot, PerplexityBot, Google-Extended, CCBot,
# anthropic-ai) are explicitly ALLOWED for Generative Engine Optimization.
# See ADR-017.
# Cloudflare "AI Audit" / "Pay Per Crawl" must remain disabled — see
# ADR-011 runbook.
User-agent: *
Allow: /

Sitemap: https://origa.uwuwu.net/sitemap.xml
```

No per-bot `Allow` lines are added. A blanket `Allow: /` under `User-agent: *`
already permits every crawler; duplicating it per bot is noise that must be
maintained as new bots appear. The comment block is the documentation, not the
directive.

### Publish `llms.txt` as the AI-facing summary

`llms.txt` (ADR-016) gives assistants a factual, brand-voice summary they can
cite: what Origa does, the four interface languages, offline/local-AI
capabilities, and that it is currently free. This is the GEO analogue of
`robots.txt`'s crawl directive — it shapes *what* gets cited, not just
*whether* crawling is allowed.

### Cloudflare "AI Audit" / "Pay Per Crawl" must stay disabled

The Cloudflare feature is incompatible with the allow-all stance: when enabled
it prepends `Disallow: /` directives for specific AI bots, overriding our
`robots.txt`. It must remain disabled per the ADR-011 runbook. This is an
operational invariant, not a code change — it is enforced by humans via the
dashboard, and the `robots.txt` comment flags it so a future operator does
not re-enable it without understanding the consequence.

## Alternatives Considered

### A1: Selectively allow citation bots, block training bots

Considered (e.g. allow `GPTBot`/`PerplexityBot`/`Google-Extended`, block
`CCBot`/`anthropic-ai`). Rejected because:

- The boundary between "citation" and "training" is porous — the same vendor
  may use one crawler for both, and the labelling changes over time.
- Maintaining a per-bot allowlist is ongoing toil that lags behind new bots.
- For a marketing surface, there is no sensitive content to protect; the
  asymmetric risk is *not being cited*, not *being trained on*.

### A2: Block all AI crawlers

Rejected. This would remove Origa from the generative index for the exact
queries it targets. The product has no paywalled or copyrighted content that
benefits from scraping protection.

### A3: Move the Cloudflare disable into code / config

Not possible. The "AI Audit" / "Pay Per Crawl" toggle lives in the Cloudflare
dashboard, not in any file we control. The runbook in ADR-011 is the only
handle; the `robots.txt` comment is a signpost to it.

## Consequences

### Positive

- Origa is crawlable and citable by every major generative answer engine.
- `llms.txt` gives assistants an accurate, on-brand summary to quote.
- The stance is documented in-repo so it survives operator turnover.

### Negative

- Public marketing copy is available for AI training with no compensation
  (accepted — the copy is already public on the web).
- The Cloudflare toggle is a manual invariant. If a future operator enables
  "AI Audit"/"Pay Per Crawl" without reading the runbook, the allow-all
  stance silently breaks until someone notices missing citations. The
  `robots.txt` comment mitigates but cannot prevent this.
- As new AI crawlers appear, the comment's bot list goes stale; it is
  illustrative, not exhaustive, and the blanket `Allow: /` is what actually
  governs.

## References

- ADR-011: URL Canonicalization Policy — documents the Cloudflare "AI Audit" /
  "Pay Per Crawl" managed-`robots.txt` injection (RC-2) and the dashboard
  runbook to disable it.
- ADR-016: Cache-Control for `browserconfig.xml` and `llms.txt` — the
  `no-cache` policy that keeps `llms.txt` fresh at the edge.
- `public/robots.txt` and `public/llms.txt`.
