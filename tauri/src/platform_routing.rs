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

#[cfg(test)]
mod tests {
    use super::*;

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
}
