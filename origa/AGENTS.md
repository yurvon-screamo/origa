# AGENTS.md - Origa Core (`origa` crate)

## Description

Business logic of the application: domain models, use cases, traits, OCR, dictionary.

## Project Structure

```text
origa/src/
├── domain/           # Domain models and errors (thiserror)
├── use_cases/        # Business logic workflows
├── traits/           # Abstracts (Repository, OCR, etc.)
├── ocr/              # NDLOCR-Lite implementation
└── dictionary/       # Linguistic module (lindera)
```

## Key Conventions

### Error Handling

```rust
// ✅ Good: thiserror for domain errors
#[derive(Debug, thiserror::Error)]
pub enum CardError {
    #[error("Card not found: {0}")]
    NotFound(ULID),
}

// ❌ Bad: unwrap in production
card.unwrap();
```

### Typing

- All public functions must have explicit types
- No `()` instead of Result where an error is possible

### Logging

- Use `tracing` for all logs

```rust
tracing::info!("Creating card {id}");
tracing::error!("Failed to process OCR: {err}");
```

## Testing

```bash
# All crate tests
cargo test -p origa

# Specific test
cargo test -p origa --test test_name

# With println output
cargo test -p origa -- --nocapture
```
