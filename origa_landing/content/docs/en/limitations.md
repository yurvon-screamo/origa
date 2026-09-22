---
title: "Known Limitations of Origa"
slug: /docs/limitations
locale: en
meta_title: "Known Limitations of Origa — Honest Boundaries"
meta_description: "Where Origa falls short: STT format limits, OCR accuracy on stylized text, no guest mode, sync requires internet, and other current boundaries."
target_keywords: ["origa limitations", "japanese learning app limits", "japanese ocr limits", "japanese stt wav only"]
lastmod: 2026-09-23
status: ready
---

<!-- markdownlint-disable-file MD025 — frontmatter `title` is metadata, not a rendered H1; the body has a single H1 by design. -->

# Known limitations

This page lists the current boundaries of Origa. Limits are not bugs. They describe what the app does not do today, so you can decide whether it fits your workflow.

## Audio transcription accepts WAV only

The speech recognition feature accepts `.wav` files. Other common formats (MP3, M4A, OGG) are not currently supported. Convert your audio before uploading. See [Capture](/docs/capture) for the full flow.

## OCR is approximate on difficult input

Optical character recognition works well on printed text and clean screenshots. It struggles with heavily stylized or decorative fonts, low-contrast or backlit photos, vertical text in unusual layouts, very small characters, and handwritten text (not supported).

Always review the recognized words before adding them as cards.

## No guest mode

You must sign in to use Origa. An account is required because progress is synced across devices. There is no offline-only or anonymous mode.

## First run requires internet

The first time you sign in, Origa downloads dictionaries and content. The first time you use OCR or speech recognition, the corresponding model downloads. After these one-time downloads, the app runs offline. See [Getting started](/docs/getting-started) for the full breakdown.

## Sync requires internet

Your progress is stored locally and synced to the server. If you are offline, progress is queued and synced when you reconnect. You cannot push progress to another device without an internet connection.

## Writing practice is animation-based

The kanji writing feature shows the correct stroke order as an animation you follow along with. It does not currently recognize free-form handwriting.

## Two-button rating

The rating after each card is binary: **Don't know** or **Know** in a regular review, **Remember** or **Don't remember** in acquaintance training. There are no intermediate options (no "hard" or "easy" grades). The FSRS scheduler uses these two signals to set intervals.

## Counters: no browse page, first batch is long

Counter suffixes (本, 枚, 匹 …) are trained as cards, and each one arrives with the full reading table on its presentation slide. There is no separate page that lists every counter in the collection yet. The first review of a counter quizzes its number bindings in one batch, which can feel long for suffixes with many irregular readings. Marking a counter as known during onboarding skips the meaning training but not the binding drills. For accounts created before counters shipped, counters are added gradually as the acquaintance pool works through them.

## Legal texts are English/Russian only

The interface and the learning content (vocabulary, phrases, grammar, kanji) are available in English, Russian, Korean and Vietnamese. So are these documentation pages. The legal texts (privacy policy, terms of use) are currently available in English and Russian only.

## Related

- [Capture (OCR and speech)](/docs/capture)
- [Getting started](/docs/getting-started)
