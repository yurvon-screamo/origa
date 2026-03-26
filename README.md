# Origa　「オリガ」

## Download

[![Windows](https://img.shields.io/badge/Windows-Installer-0078D4?style=for-the-badge&logo=windows&logoColor=white)](https://github.com/yurvon-screamo/origa/releases/latest/download/origa_latest_x64-setup.exe)
[![Linux](https://img.shields.io/badge/Linux-AppImage-FCC624?style=for-the-badge&logo=linux&logoColor=black)](https://github.com/yurvon-screamo/origa/releases/latest/download/origa_latest_amd64.AppImage)
[![macOS](https://img.shields.io/badge/macOS-Apple_Silicon-000000?style=for-the-badge&logo=apple&logoColor=white)](https://github.com/yurvon-screamo/origa/releases/latest/download/Origa-macos-arm64.zip)
[![Android](https://img.shields.io/badge/Android-APK-3DDC84?style=for-the-badge&logo=android&logoColor=white)](https://github.com/yurvon-screamo/origa/releases/latest/download/origa-latest.apk)

[📥 All releases](https://github.com/yurvon-screamo/origa/releases)

Origa「オリガ」 - application for learning japanese. Like Anki, but for japanese native and simplified for better user experience.

Learn all flashcards as a single set with FSRS algorithm, tracking your Japanese language progress as single value.

Builded with Rust only.

## Supported native languages

* Russian
* English
* [TODO: 1 priority] Vietnamese
* [TODO: 2 priority] Korean
* [TODO: 2 priority] Indonesian

## Development

This project uses cargo-make as a task runner for common operations.

### Install cargo-make

```bash
cargo install cargo-make
```

### Common Commands

```bash
# Show all available tasks
cargo make

# Development
cargo make dev              # Start frontend dev server
cargo make dev-tauri        # Start Tauri desktop app

# Building
cargo make build            # Build all workspace (debug)
cargo make build-release    # Build all workspace (release)
cargo make build-ui         # Build frontend for production
cargo make build-tauri      # Build Tauri desktop application

# Testing
cargo make test             # Run all workspace tests
cargo make test-verbose     # Run tests with output
cargo make test-cov         # Generate test coverage report (HTML)

# Code Quality
cargo make lint             # Run all linting checks (fmt + clippy)
cargo make fmt              # Format code
cargo make clippy           # Run clippy linter

# E2E Testing
cargo make e2e              # Run E2E tests (headless)
cargo make e2e-headed       # Run E2E tests in visible browser
cargo make e2e-ui           # Run E2E tests with Playwright UI

# CI/CD
cargo make ci               # Run full CI pipeline (lint + test)
cargo make ci-full          # Run full CI with E2E and qlty

# Utilities
cargo make clean            # Clean all build artifacts
cargo make check            # Check workspace for errors
cargo make docs             # Generate and open documentation
cargo make deps             # Show dependency tree

# Pre-commit hook
cargo make pre-commit       # Run fmt + clippy + test
```

For full list of available tasks, see [cargo-make.toml](origa/cargo-make.toml).

## Features

(partially implemented)

* Learning vocabulary with FSRS algorithm
* Learning grammar with FSRS algorithm
* Learning kanji with FSRS algorithm
* Automatic JLPT level detection based on learned content (kanji, vocabulary, grammar)
* Visual progress tracking for each JLPT level (N5-N1) with detailed breakdown
* Generation of translation for the flashcards
* Generation of furigana for the text
* Extraction kanji from the flashcards
* Show radicals for the kanji
* Import flashcards from anki
* Import flashcards from screenshots or any images with text
* Import flashcards from youtube videos
* Import flashcards from migii
* Sync flashcards list from duolingo
* Tokenize and deduplicate vocabulary flashcards
* Ready collection of flashcards for each level of Japanese language

## License

Application is licensed under [Business Source License 1.1](./LICENSE).
