//! Runtime platform identity for pure per-platform routing decisions.
//!
//! Platform-dependent behaviour (update policy, store review support) is
//! implemented as pure functions over this enum, so an rstest table can pin
//! the whole decision matrix on a single host runner without any device
//! (task 20261104060604, GH #854: the #701 class — a feature dead on one
//! platform behind a green CI). Production code converts `cfg!()` into the
//! enum exactly once, via [`HostPlatform::current`]; everything downstream
//! is cfg-free logic that every host CI can see.

/// The platforms the `tauri-app` crate is ever compiled for (see the
/// workspace release targets and the CI build matrix).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HostPlatform {
    Windows,
    MacOS,
    Linux,
    Android,
    Ios,
}

impl HostPlatform {
    /// The platform this binary is being compiled for. The five arms cover
    /// every target the workspace builds the tauri crate for; the trailing
    /// arm keeps the function total for any exotic future target rather
    /// than introducing a panic path.
    pub fn current() -> Self {
        if cfg!(windows) {
            Self::Windows
        } else if cfg!(target_os = "macos") {
            Self::MacOS
        } else if cfg!(target_os = "linux") {
            Self::Linux
        } else if cfg!(target_os = "android") {
            Self::Android
        } else {
            // Not Windows/macOS/linux/android and still compiling Rust for a
            // Tauri shell — the only remaining supported shell is iOS.
            Self::Ios
        }
    }
}

/// Whether the `tauri-plugin-tts` registration on a platform requires a
/// successful speech-engine probe ([`TtsRegistration::ProbeRequired`]) or
/// registers the plugin unconditionally ([`TtsRegistration::Always`]).
///
/// The plugin eagerly connects to the speech engine inside its setup: a
/// failed engine init kills the whole `Builder::run` with
/// `PluginInitialization` before any window appears (Linux: missing
/// speech-dispatcher daemon, ee1f4838; Windows: WinRT error — Sentry
/// RUST-S, Win10 0.7.37). Probe-gated platforms run `tts::Tts::default()`
/// first — the exact engine init the plugin performs — and keep the app
/// alive without TTS when it fails (the frontend already degrades
/// gracefully: `plugin:tts|speak` invoke rejects into a `tracing::warn`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TtsRegistration {
    /// Probe the engine first; register the plugin only when it initializes.
    ProbeRequired,
    /// Register the plugin unconditionally.
    Always,
}

/// The TTS registration decision table (ADR-086 layer 1: a pure table over
/// [`HostPlatform`], pinned by the host rstest matrix in this file).
/// Desktop platforms with a known engine-init failure mode are probe-gated
/// (Linux: speech-dispatcher may be absent; Windows: WinRT speech init can
/// fail on the user's machine); macOS and the mobile native bridges have
/// no observed failures and register unconditionally. A new
/// `HostPlatform` variant fails to compile here until it picks a cell.
pub fn tts_registration(platform: HostPlatform) -> TtsRegistration {
    match platform {
        HostPlatform::Linux | HostPlatform::Windows => TtsRegistration::ProbeRequired,
        HostPlatform::MacOS | HostPlatform::Android | HostPlatform::Ios => TtsRegistration::Always,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    /// `current()` must map every host compilation to the platform actually
    /// compiling the crate (Linux CI asserts the Linux arm; each release
    /// target asserts its own arm when its job compiles the tests).
    #[test]
    fn current_matches_the_compiling_target() {
        let expected = if cfg!(windows) {
            HostPlatform::Windows
        } else if cfg!(target_os = "macos") {
            HostPlatform::MacOS
        } else if cfg!(target_os = "android") {
            HostPlatform::Android
        } else if cfg!(target_os = "ios") {
            HostPlatform::Ios
        } else {
            HostPlatform::Linux
        };
        assert_eq!(HostPlatform::current(), expected);
    }

    /// The whole TTS registration table is pinned on the host (ADR-086
    /// layer 1). Windows must be probe-gated: an unconditional
    /// `tauri-plugin-tts` registration dies with `PluginInitialization`
    /// when the WinRT speech engine fails to init (Sentry RUST-S — the app
    /// never shows a window). Five cells cover every `HostPlatform`
    /// variant; a sixth variant breaks the exhaustive match in
    /// `tts_registration` instead of silently registering unconditionally.
    #[rstest]
    #[case(HostPlatform::Windows, TtsRegistration::ProbeRequired)]
    #[case(HostPlatform::Linux, TtsRegistration::ProbeRequired)]
    #[case(HostPlatform::MacOS, TtsRegistration::Always)]
    #[case(HostPlatform::Android, TtsRegistration::Always)]
    #[case(HostPlatform::Ios, TtsRegistration::Always)]
    fn tts_registration_table_is_pinned(
        #[case] platform: HostPlatform,
        #[case] expected: TtsRegistration,
    ) {
        assert_eq!(tts_registration(platform), expected);
    }
}
