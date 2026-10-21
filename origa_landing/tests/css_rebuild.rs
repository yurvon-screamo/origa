//! Prove-It tests for `origa_landing/css_rebuild.rs` (issue #832).
//!
//! `#[path]`-includes the pure decision module and pins the full matrix:
//! installed Tailwind always rebuilds regardless of the stylesheet; without
//! it a present stylesheet keeps the build green (the production Docker path)
//! and a missing one must fail — a binary without the stylesheet would serve
//! 404 on `/landing.processed.css`.

#[path = "../css_rebuild.rs"]
mod css_rebuild;

use css_rebuild::{CssAction, css_rebuild_action};
use rstest::rstest;

#[rstest]
#[case(true, true, CssAction::Rebuild)]
#[case(true, false, CssAction::Rebuild)]
#[case(false, true, CssAction::UseExisting)]
#[case(false, false, CssAction::Fail)]
fn css_rebuild_action_pins_the_full_matrix(
    #[case] tailwind_installed: bool,
    #[case] css_output_exists: bool,
    #[case] expected: CssAction,
) {
    assert_eq!(
        css_rebuild_action(tailwind_installed, css_output_exists),
        expected
    );
}
