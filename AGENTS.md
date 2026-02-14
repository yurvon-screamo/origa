# AGENTS.md

This document provides guidelines for agentic coding assistants working in this repository.

## Project Overview

Origa is a Japanese language learning application built with Rust.

Workspace structure:

- `origa/` - Core domain and application logic library
- `origa_ui/` - Leptos UI library with Tauri bindings
- `tokenizer/` - Japanese text tokenization service
- `tauri/` - Tauri desktop application binary
- `telegram/` - Telegram desktop application binary

## Code Style Guidelines

### General Principles

- Write concise, focused functions under 50 lines when possible
- Use early returns to reduce nesting
- Prefer composition over inheritance (Rust idiomatic)
- Avoid unnecessary clones; use references or Arc/Rc where appropriate

### Error Handling

- Define errors as enums with Display and Error traits
- Use `thiserror` for simpler error definitions when appropriate
- Propagate errors with `?` operator
- Include context in error messages (what failed, why, what values)
- Avoid generic error types; use specific error variants

### Async/Await Patterns

- Prefer Result<T, E> over Option for async operations
- Use tokio::spawn for parallel operations when appropriate
- Use `async-trait` for trait methods that need to be async
- Avoid blocking in async code; use async equivalents
- Handle errors explicitly; don't silently ignore them
