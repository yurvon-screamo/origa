//! Stdout-channel guard for every workspace build script (task #1483).
//!
//! Cargo parses `cargo:` directives (warnings, rerun triggers, rustc-env
//! injection) from a build script's **stdout** only; stderr is diverted to
//! the sidecar `target/{profile}/build/<pkg>-<hash>/stderr`, which no one
//! reads in a default `cargo build`/CI run (it surfaces only under `-vv`).
//! A `cargo:warning=...` line printed with `eprintln!` is therefore a
//! **silent** warning: the build operator never sees it. That exact bug
//! shipped in `origa_ui/build.rs` — the invalid-duolingo-set skip warning
//! (`cargo:warning=Skipping duolingo set: ...`) sat on `eprintln!` for
//! months, so CDNs kept receiving metadata without the skipped set and
//! nobody could tell (task #1483, PR #964).
//!
//! This suite pins the contract mechanically: no build script in the
//! workspace may print a `cargo:` directive through `eprintln!`. It scans
//! every crate-root `build.rs` for the literal pattern `eprintln!(` +
//! (whitespace) + `"cargo:` — the bug shape, in both its single-line and
//! rustfmt-wrapped forms. Genuine stderr diagnostics (messages that do not
//! start with `cargo:`) stay legal: only the directive channel is guarded,
//! not `eprintln!` itself.

#![cfg(not(target_arch = "wasm32"))] // std::fs path walking (wasm-pack compiles this suite too)

use std::fs;
use std::path::{Path, PathBuf};

/// Workspace root: the parent of this crate's directory (origa_ui sits
/// directly under it). Fails loudly if the layout ever changes instead of
/// silently scanning nothing.
fn workspace_root() -> PathBuf {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("origa_ui has a parent")
        .to_path_buf();
    assert!(
        root.join("Cargo.toml").is_file(),
        "workspace root with Cargo.toml not found from {}",
        env!("CARGO_MANIFEST_DIR")
    );
    root
}

/// All crate-root `build.rs` scripts of the workspace: the workspace root
/// itself plus one level of crate directories (origa, origa_ui,
/// origa_landing, tts, tauri, ...). A crate nested deeper than one level
/// with its own build script would be missed here — add it explicitly if
/// such a crate ever appears.
fn workspace_build_scripts() -> Vec<PathBuf> {
    let root = workspace_root();
    let mut scripts = Vec::new();
    let mut push_if_exists = |path: PathBuf| {
        if path.is_file() {
            scripts.push(path);
        }
    };
    push_if_exists(root.join("build.rs"));
    let Ok(entries) = fs::read_dir(&root) else {
        panic!("workspace root {} is readable", root.display());
    };
    let mut crates: Vec<PathBuf> = entries
        .flatten()
        .filter(|e| e.file_type().map(|t| t.is_dir()).unwrap_or(false))
        .map(|e| e.path())
        .collect();
    crates.sort();
    for dir in crates {
        push_if_exists(dir.join("build.rs"));
    }
    scripts
}

/// True when the source text contains `eprintln!(` whose first string
/// literal argument starts with `cargo:` — a directive routed to stderr.
/// Handles both `eprintln!("cargo:...` and the rustfmt-wrapped
/// `eprintln!(\n    "cargo:...` form; whitespace between the paren and the
/// literal is insignificant.
fn prints_cargo_directive_to_stderr(source: &str) -> bool {
    let mut rest = source;
    while let Some(pos) = rest.find("eprintln!(") {
        let after = rest[pos + "eprintln!(".len()..]
            .trim_start()
            .trim_start_matches(['\n', '\r', ' ', '\t']);
        if after.starts_with("\"cargo:") {
            return true;
        }
        rest = &rest[pos + "eprintln!(".len()..];
    }
    false
}

#[test]
fn no_build_script_prints_cargo_directives_to_stderr() {
    let scripts = workspace_build_scripts();
    assert!(
        !scripts.is_empty(),
        "no build.rs found under {} — the scan silently guarded nothing",
        workspace_root().display()
    );
    for script in &scripts {
        let source =
            fs::read_to_string(script).unwrap_or_else(|e| panic!("read {}: {e}", script.display()));
        assert!(
            !prints_cargo_directive_to_stderr(&source),
            "{} prints a `cargo:` directive via eprintln! — cargo parses \
             directives from the build script's stdout only, so this warning \
             is silently swallowed (stderr lands in target/*/build/*/stderr, \
             invisible in a default build). Route it through println! like \
             every other cargo: directive in the file (task #1483)",
            script.display()
        );
    }
}

#[cfg(test)]
mod detector_tests {
    use super::prints_cargo_directive_to_stderr;

    #[test]
    fn detects_single_line_form() {
        assert!(prints_cargo_directive_to_stderr(
            r#"eprintln!("cargo:warning=skipped");"#
        ));
    }

    #[test]
    fn detects_rustfmt_wrapped_form() {
        assert!(prints_cargo_directive_to_stderr(
            "eprintln!(\n                \"cargo:warning=Skipping duolingo set: invalid level {:?}\",\n                level\n            );"
        ));
    }

    #[test]
    fn ignores_plain_stderr_diagnostics() {
        assert!(!prints_cargo_directive_to_stderr(
            r#"eprintln!("progress: {} items done", n);"#
        ));
        assert!(!prints_cargo_directive_to_stderr(
            r#"eprintln!("cargo not found in PATH");"#
        ));
    }

    #[test]
    fn ignores_stdout_directives() {
        assert!(!prints_cargo_directive_to_stderr(
            r#"println!("cargo:warning=ok");"#
        ));
    }
}
