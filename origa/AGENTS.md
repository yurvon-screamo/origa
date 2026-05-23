# AGENTS.md — Origa Core (`origa` crate)

Core business logic: domain models, use cases, traits, OCR, STT, dictionary. Rust edition 2024.

## Project Structure

```text
origa/src/
├── domain/
│   ├── error.rs            # OrigaError enum + ErrorCategory
│   ├── srs.rs              # FSRS spaced repetition
│   ├── knowledge/          # Card, Vocabulary, Kanji, Grammar, Phrase, Lesson, Stats
│   ├── memory/             # SRS memory state (value objects)
│   ├── tokenizer/          # Part-of-speech, translation domain types
│   └── grammar/            # Grammar forms + quiz generation
├── use_cases/              # ~20 business logic workflows
├── traits/                 # UserRepository, CdnProvider trait definitions
├── ocr/                    # NDLOCR-Lite pipeline (ONNX)
├── stt/                    # Whisper-based speech-to-text (ONNX)
└── dictionary/             # Furigana, grammar, kanji, phrase, vocabulary modules
```

## Error Handling

Single `OrigaError` enum (~40 variants) via `thiserror` 2.0, mapped to `ErrorCategory` (Domain / Infrastructure / Import).

```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, thiserror::Error)]
pub enum OrigaError {
    #[error("Card with id {card_id} not found")]
    CardNotFound { card_id: Ulid },
    #[error("OCR processing failed: {reason}")]
    OcrFailed { reason: String },
}
```

Never `unwrap()` in production. Classify via `.category()` for UI handling.

## Conditional Compilation

Native vs WASM via `cfg(target_arch = "wasm32")`. Each module has `*.rs` (native) and `*_wasm.rs` counterparts. Native: `rusqlite`, `hound`. WASM: `ort` + `ort-web`.

## Key Dependencies

- **rs-fsrs** — spaced repetition algorithm
- **lindera** + UniDic — Japanese tokenization
- **ort** — ONNX Runtime (OCR + STT inference)
- **rkyv** — zero-copy dictionary deserialization
- **rusqlite** — SQLite for Anki import (native only)
- **ulid** — unique identifiers everywhere

## Conventions

- **IDs**: always `Ulid` — never raw strings or integers
- **Logging**: `tracing` only — never `println!`
- **Async**: `async fn` directly — never `#[async_trait]`
- **Dead code**: never `#[allow(dead_code)]`
- **Tests**: `rstest` for parameterized cases
- **Types**: explicit signatures on all public functions

## Testing

```bash
cargo test -p origa                    # All tests
cargo test -p origa test_name          # Specific test
cargo test -p origa -- --nocapture     # With output
```

## Boundaries

**Always:** `cargo clippy -p origa -- -D warnings` + `cargo fmt` + all tests green before commit.
**Ask First:** changes to `domain/` or `Cargo.toml`.
**Never:** `unwrap()` in production, `#[async_trait]`, `#[allow(dead_code)]`, `println!`/`console.log`, removing tests.
