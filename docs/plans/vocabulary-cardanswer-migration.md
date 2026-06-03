# Plan: CardAnswer Enum — Vocabulary Domain API Migration

## Overview

Replace `Answer { text: String }` with `CardAnswer` enum. Domain returns structured data only — no formatting methods. Each consumer handles presentation.

## New Type

```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CardAnswer {
    Vocabulary { translations: Vec<String>, description: Option<String> },
    Text(String),
}

impl CardAnswer {
    pub fn text(s: String) -> Result<Self, OrigaError> {
        let trimmed = s.trim().to_string();
        if trimmed.is_empty() { return Err(OrigaError::InvalidAnswer { reason: "empty".into() }); }
        Ok(CardAnswer::Text(trimmed))
    }
    
    pub fn vocabulary(translations: Vec<String>, description: Option<String>) -> Result<Self, OrigaError> {
        if translations.is_empty() { return Err(OrigaError::InvalidAnswer { reason: "no translations".into() }); }
        Ok(CardAnswer::Vocabulary { translations, description })
    }
}
```

## Key Decisions

- `CardAnswer::text()` constructor validates non-empty (preserves Answer::new() guard)
- `OrigaError::InvalidAnswer` variant KEPT — used by CardAnswer::text()
- reverse_side → `CardAnswer::Text` (single word, not structured)
- `get_translation()` legacy method KEPT — computes markdown from structured data, used by `KanjiCard::example_words()` and `VocabularyCard::validate_translation()`
- Lesson quiz format CHANGES: from markdown bullets to comma-separated translations (explicit decision)
- `PhraseCard.answer()` returns `Option<String>` — NOT changed. Card dispatch wraps in `CardAnswer::Text`.
- `phrase_card_item.rs` does NOT need changes — calls `PhraseCard::answer()`, not `Card::answer()`
- `GrammarRuleCard::description()` has UNIQUE implementation (not macro) — must update separately
- Macro `get_content!` needs to handle both `Question::new` and `CardAnswer::text` return types
- CDN deployment: LOCAL ONLY, user deploys to CDN at the end
- `Hash` derive: NOT needed currently, remove from NFRs

## Deletion Checklist

- `value_objects.rs` Answer struct + impl + tests
- `domain/mod.rs` Answer re-export → CardAnswer re-export
- All `use crate::domain::Answer` → `use crate::domain::CardAnswer`

## CDN Format Migration

Current: `{ "猫": { "level": "N5", "russian_translation": "- кошка\n- кот", "english_translation": "- cat" } }`
Target: `{ "猫": { "level": "N5", "ru": { "t": ["кошка", "кот"], "d": "" }, "en": { "t": ["cat"], "d": "" } } }`

## VocabularyInfo New Structure

```rust
struct VocabularyInfo {
    word: String,
    ru_translations: Vec<String>,
    ru_description: Option<String>,
    en_translations: Vec<String>,
    en_description: Option<String>,
}
```

- `get_translation()` → legacy, computes `"- " + join("\n- ")` from translations
- `get_translations()` → `Vec<String>` (new)
- `get_description()` → `Option<String>` (new)

## Complete Blast Radius

### Domain (origa/src/domain/)

- `value_objects.rs` — Answer deleted, CardAnswer added
- `knowledge/card.rs` — Card::answer() → CardAnswer dispatch
- `knowledge/vocabulary.rs` — answer(), revert(), with_grammar_rule()
- `knowledge/kanji.rs` — description() → CardAnswer::Text, example_words() uses get_translation() (no change)
- `knowledge/grammar.rs` — description() (unique impl!), short_description(), explanation(), how_to_form(), examples(), nuances(), pro_tip() — all via macro or direct
- `knowledge/phrase.rs` — NO CHANGE (returns Option<String>)

### Dictionary (origa/src/dictionary/)

- `vocabulary.rs` — VocabularyEntryStoredType, VocabularyInfo, get_translation(), new get_translations(), get_description()

### UI Business Logic

- `pages/shared/card_list_page.rs` — search pattern match
- `pages/kanji/content.rs` — search CardAnswer::Text

### UI Consumers

- `pages/words/vocabulary_card_item.rs` — WordTranslations component
- `pages/lesson/quiz_options_multi.rs` — pattern match
- `pages/lesson/quiz_options.rs` — pattern match
- `pages/lesson/lesson_card_answer.rs` — pattern match
- `pages/lesson/yesno_card_view.rs` — pattern match
- `pages/lesson/lesson_card_container.rs` — pattern match
- `pages/home/recent_study.rs` — pattern match
- `pages/home/dashboard_stats.rs` — pattern match
- `pages/onboarding/scoring_helpers.rs` — pattern match
- `pages/onboarding/scoring_step.rs` — pattern match
- `pages/kanji/kanji_card_item.rs` — CardAnswer::Text
- `pages/kanji/kanji_detail.rs` — CardAnswer::Text
- NOTE: `pages/phrases/phrase_card_item.rs` — NO CHANGE

### Lesson Generation

- `origa/src/domain/knowledge/lesson/view_generator/generation.rs` — 6 call sites, pattern match

### Tests (~20 files in origa/, ~5 in origa_ui/)

- All tests using `.answer().unwrap().text()` → pattern match on CardAnswer variants

### CDN Data

- `cdn/dictionary/chunk_01.json` ... `chunk_11.json` — format migration (local only)

## Slices

### Slice-1 (parallel): Migration script + CardAnswer enum

- `scripts/migrate_vocabulary_format.py` — migrate CDN JSON data
- `origa/src/domain/value_objects.rs` — CardAnswer enum, delete Answer

### Slice-2: CDN parser update

- `origa/src/dictionary/vocabulary.rs` — untagged enum, structured VocabularyInfo

### Slice-3: Domain model update

- `card.rs`, `vocabulary.rs`, `kanji.rs`, `grammar.rs`, all domain tests

### Slice-4: Business logic consumers

- `card_list_page.rs`, `kanji/content.rs`, `scoring_helpers.rs`, `dashboard_stats.rs`, lesson generation

### Slice-5: UI consumers

- All lesson UI, home, onboarding, kanji card/detail

### Slice-6: WordTranslations + vocabulary card

- `word_translations.rs` (new), `vocabulary_card_item.rs`, CSS

### Slice-7: E2E + final gate

- cargo test, clippy, fmt, playwright
