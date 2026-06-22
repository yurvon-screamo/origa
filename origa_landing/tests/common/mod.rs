//! Shared helpers for `origa_landing` integration tests.
//!
//! Each file in `tests/` compiles as a separate integration-test binary, so
//! code reused across them must live in a module rather than a `#[path]`
//! import. `tests/common/mod.rs` is the conventional location and is NOT
//! picked up by Cargo as its own test target (only direct children of
//! `tests/` are). Each test file pulls it in with `mod common;`.

use leptos::config::LeptosOptions;

/// Build the exact same Axum router the production binary serves, so tests
/// exercise the real middleware stack (`strip_trailing_slash`,
/// `enforce_cache_policy`) and the real route table.
pub fn test_router() -> axum::Router {
    let opts = LeptosOptions::builder()
        .output_name("origa_landing")
        .build();
    origa_landing::server::build_router(opts)
}
