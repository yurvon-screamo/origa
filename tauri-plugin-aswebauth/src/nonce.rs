// Copyright 2026 yurvon-screamo
// SPDX-License-Identifier: MIT

//! Anti-replay nonce for the native Sign in with Apple flow.
//!
//! Platform-independent on purpose: both the macOS implementation (`siwa.rs`)
//! and the iOS command (which passes the values to the Swift plugin) use this
//! module, so the nonce contract lives in exactly one place.
//!
//! Contract (shared with the TrailBase endpoint and pinned by unit tests on
//! every platform): the raw nonce is 32 random bytes base64url-encoded without
//! padding; `request.nonce` = lowercase-hex SHA-256 of the raw string's UTF-8
//! bytes. Known-answer vector: `sha256_hex("test") ==
//! "9f86d081884c7d659a2feaa0c55ad015a3bf4f1b2b0b822cd15d6c15b0f00a08"`.

use base64::Engine as _;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use rand::RngCore;
use sha2::{Digest, Sha256};

/// Generates the raw client nonce: 32 random bytes, base64url-encoded without
/// padding (RFC 4648 §5, un-padded — same alphabet as PKCE verifiers).
pub(crate) fn generate_raw_nonce() -> String {
    let mut bytes = [0u8; 32];
    rand::rng().fill_bytes(&mut bytes);
    URL_SAFE_NO_PAD.encode(bytes)
}

/// Lowercase-hex SHA-256 of the raw nonce string's UTF-8 bytes.
pub(crate) fn sha256_hex(value: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(value.as_bytes());
    hasher
        .finalize()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The nonce hash is a cross-platform contract (macOS Rust, iOS Swift
    /// consumer, TrailBase endpoint): pin the documented known-answer vector.
    #[test]
    fn sha256_hex_matches_the_cross_platform_test_vector() {
        assert_eq!(
            sha256_hex("test"),
            "9f86d081884c7d659a2feaa0c55ad015a3bf4f1b2b0b822cd15d6c15b0f00a08"
        );
    }

    /// The raw nonce must be valid base64url without padding: 32 bytes
    /// encode to exactly 43 characters.
    #[test]
    fn generated_nonce_is_43_char_base64url() {
        let nonce = generate_raw_nonce();
        assert_eq!(nonce.len(), 43);
        assert!(!nonce.contains('+') && !nonce.contains('/') && !nonce.contains('='));
    }
}
