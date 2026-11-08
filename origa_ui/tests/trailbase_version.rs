//! Drift guard for the TrailBase runtime version pinned in `origa_ui/Dockerfile`.
//!
//! TrailBase v0.33.3 introduced a memory regression (steady-state RSS ~0.6 GB
//! at near-zero load — the largest memory consumer of the whole Railway
//! project); upstream fixed it in v0.34.0 ("Fixed significant regression
//! introduced in v0.33.3"). The `ARG TRAILBASE_VERSION` line is the only thing
//! keeping the deployment out of that regression window: an accidental revert
//! or a "rollback" that copies the old line back silently re-inflates the
//! footprint (spike task 20261106222807, PR #891). This test fails first and
//! points at the spike instead of letting the change ship unnoticed.

use rstest::rstest;

const DOCKERFILE: &str = include_str!("../Dockerfile");

/// First release without the v0.33.3 memory regression (upstream release
/// notes, v0.34.0).
const MEMORY_FIX: (u32, u32, u32) = (0, 34, 0);

/// Parse a `vMAJOR.MINOR.PATCH` release tag.
fn parse_version(raw: &str) -> Result<(u32, u32, u32), String> {
    let digits = raw
        .strip_prefix('v')
        .ok_or_else(|| format!("missing `v` prefix: {raw:?}"))?;
    let mut parts = digits.split('.');
    let mut out = [0u32; 3];
    for (i, slot) in out.iter_mut().enumerate() {
        let part = parts
            .next()
            .ok_or_else(|| format!("missing component {i}: {raw:?}"))?;
        *slot = part
            .parse()
            .map_err(|_| format!("non-numeric component {i}: {raw:?}"))?;
    }
    if parts.next().is_some() {
        return Err(format!("more than three components: {raw:?}"));
    }
    Ok((out[0], out[1], out[2]))
}

/// The single `ARG TRAILBASE_VERSION=…` value of the runtime stage.
fn pinned_trailbase_version() -> String {
    let needle = "ARG TRAILBASE_VERSION=";
    let occurrences = DOCKERFILE.matches(needle).count();
    assert_eq!(
        occurrences, 1,
        "the runtime stage must pin TRAILBASE_VERSION exactly once, got {occurrences}"
    );
    let line = DOCKERFILE
        .lines()
        .find(|l| l.starts_with(needle))
        .expect("occurrence checked above");
    line[needle.len()..].trim().to_string()
}

#[rstest]
#[case("v0.33.16", (0, 33, 16))]
#[case("v0.34.0", (0, 34, 0))]
#[case("v0.35.10", (0, 35, 10))]
fn parses_release_tags(#[case] raw: &str, #[case] expected: (u32, u32, u32)) {
    assert_eq!(parse_version(raw), Ok(expected));
}

#[rstest]
#[case("0.34.0")]
#[case("v0.34")]
#[case("v0.34.0.1")]
#[case("vX.Y.Z")]
fn rejects_malformed_versions(#[case] raw: &str) {
    assert!(parse_version(raw).is_err(), "{raw:?} must not parse");
}

#[test]
fn dockerfile_pins_trailbase_at_or_above_the_memory_fix() {
    let pinned = pinned_trailbase_version();
    let version = parse_version(&pinned)
        .unwrap_or_else(|e| panic!("malformed TRAILBASE_VERSION {pinned:?}: {e}"));
    assert!(
        version >= MEMORY_FIX,
        "TRAILBASE_VERSION {pinned} sits inside the v0.33.3–v0.33.x memory \
         regression window (steady-state RSS ~0.6 GB at near-zero load); the \
         fix landed upstream in v0.34.0 — bump the ARG (task 20261106222807)"
    );
}
