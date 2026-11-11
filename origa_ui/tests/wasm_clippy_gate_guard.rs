//! Gate guard for the wasm-target clippy step in the CI `Lint` job (task
//! 20261109191924-oaxdtvh).
//!
//! The native clippy step is blind to `target_arch = "wasm32"` code: the
//! wasm-only modules and the `*_wasm_tests.rs` suites never compile on a host
//! target, so wasm-only lint debt was invisible to every gate and had to be
//! re-attributed by hand on each wasm-touching PR (#905, #903). The fix — a
//! `Run wasm clippy` step inside the `Lint` job — only works while the step
//! stays there: if it is deleted, moved to a job outside the CI gate, or run
//! without `-D warnings`, debt silently re-accumulates with no failing check
//! anywhere. These tests pin the step (exact command) inside the `Lint` job
//! block of `.github/workflows/ci.yml`, so removing or weakening the gate
//! turns this suite red instead.

#![cfg(not(target_arch = "wasm32"))] // reads repo files; wasm-pack compiles this suite too

use std::fs;
use std::path::Path;

/// The gate exactly as it must be wired: the UI crate on the real wasm target,
/// all targets (the debt lives in lib-test code), the csr feature the wasm
/// test build uses, and warnings denied.
const GATE_COMMAND: &str = "cargo clippy -p origa_ui --target wasm32-unknown-unknown --all-targets --features csr -- -D warnings";

/// Body of the `lint:` job from `.github/workflows/ci.yml` (from its key to
/// the next top-level job key).
fn lint_job_block() -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join(".github")
        .join("workflows")
        .join("ci.yml");
    let text = fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    let lines: Vec<&str> = text.lines().collect();
    let start = lines
        .iter()
        .position(|l| *l == "    lint:")
        .unwrap_or_else(|| panic!("`lint:` job not found in {}", path.display()));
    let end_rel = lines[start + 1..]
        .iter()
        .position(|l| is_top_level_job_key(l))
        .unwrap_or_else(|| panic!("no job follows `lint:` in {}", path.display()));
    lines[start..=start + 1 + end_rel].join("\n")
}

/// Top-level job keys live at a 4-space indent in ci.yml (`    lint:`); job
/// bodies start at 8. A line indented exactly 4 spaces containing a `:` is a
/// job key.
fn is_top_level_job_key(line: &str) -> bool {
    line.starts_with("    ")
        && !line.starts_with("     ")
        && line.trim_end() != "    "
        && line[4..].contains(':')
}

#[test]
fn wasm_clippy_step_stays_inside_the_lint_job() {
    let block = lint_job_block();
    assert!(
        block.contains("- name: Run wasm clippy"),
        "the `Run wasm clippy` step is gone from the Lint job — the only job wired \
         into the CI gate. Without it wasm-only lint debt is invisible to every check; \
         restore the step (see task 20261109191924-oaxdtvh)"
    );
}

#[test]
fn wasm_clippy_step_pins_target_features_and_denies_warnings() {
    let block = lint_job_block();
    assert!(
        block.contains(GATE_COMMAND),
        "the wasm clippy step no longer runs `{GATE_COMMAND}`. A weaker form \
         (different target, missing --all-targets/--features csr, or warnings \
         not denied) re-opens the debt this gate closes; restore the exact command"
    );
}
