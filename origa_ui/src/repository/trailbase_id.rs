//! Conversion between the TrailBase user id (JWT `sub`) and the `Ulid` used
//! throughout the app as the canonical user identity.
//!
//! TrailBase encodes the user UUID as url-safe base64, but to stay robust across
//! server/auth-provider changes this module accepts three encodings in order:
//! url-safe base64 without padding, url-safe base64 with padding, and a
//! canonical hex UUID with dashes. Any input that does not decode to exactly
//! 16 bytes fails safe to `Ulid::nil()` and emits a warning so callers can guard
//! against the nil value (nil is the bug state, not a valid identity).

use base64::Engine;
use base64::engine::general_purpose::{URL_SAFE, URL_SAFE_NO_PAD};
use ulid::Ulid;

/// Decode a TrailBase user id from a JWT `sub` claim into a `Ulid`.
///
/// Order tried: url-safe base64 no-pad, url-safe base64 padded, hex UUID with
/// dashes. Returns `Ulid::nil()` (and warns) when no encoding yields 16 bytes;
/// callers must treat nil as "unknown identity" and avoid propagating it.
pub(crate) fn uuid_to_ulid(b64_uuid: &str) -> Ulid {
    if b64_uuid.is_empty() {
        tracing::warn!("Empty trailbase_id, using nil ULID");
        return Ulid::nil();
    }

    if let Some(ulid) = decode_base64_to_ulid(&URL_SAFE_NO_PAD, b64_uuid) {
        return ulid;
    }
    if let Some(ulid) = decode_base64_to_ulid(&URL_SAFE, b64_uuid) {
        return ulid;
    }
    if let Some(ulid) = decode_hex_uuid_to_ulid(b64_uuid) {
        return ulid;
    }

    tracing::warn!(
        "Invalid trailbase_id format (expected 16-byte base64 or hex UUID), using nil ULID: {}",
        redact(b64_uuid)
    );
    Ulid::nil()
}

fn decode_base64_to_ulid(engine: &impl Engine, input: &str) -> Option<Ulid> {
    let bytes = engine.decode(input).ok()?;
    bytes_to_ulid(&bytes)
}

fn decode_hex_uuid_to_ulid(input: &str) -> Option<Ulid> {
    let stripped = input.replace('-', "");
    if stripped.len() != 32 {
        return None;
    }
    let mut bytes = [0u8; 16];
    for (i, chunk) in stripped.as_bytes().chunks(2).enumerate() {
        let hex = std::str::from_utf8(chunk).ok()?;
        bytes[i] = u8::from_str_radix(hex, 16).ok()?;
    }
    Some(Ulid::from_bytes(bytes))
}

fn bytes_to_ulid(bytes: &[u8]) -> Option<Ulid> {
    if bytes.len() != 16 {
        return None;
    }
    let mut arr = [0u8; 16];
    arr.copy_from_slice(bytes);
    Some(Ulid::from_bytes(arr))
}

/// Reduce an identifier before logging so full user ids do not land in logs.
pub(crate) fn redact(id: &str) -> String {
    let len = id.chars().count();
    if len <= 8 {
        return "***".to_string();
    }
    let first: String = id.chars().take(4).collect();
    let last: String = id.chars().skip(len.saturating_sub(4)).collect();
    format!("{first}…{last} ({len} chars)")
}

#[cfg(test)]
mod tests {
    use super::*;
    use base64::engine::general_purpose::URL_SAFE_NO_PAD;

    fn encode_ulid_as_base64_url_no_pad(ulid: Ulid) -> String {
        URL_SAFE_NO_PAD.encode(ulid.to_bytes())
    }

    fn encode_ulid_as_base64_url_padded(ulid: Ulid) -> String {
        URL_SAFE.encode(ulid.to_bytes())
    }

    fn encode_ulid_as_hex_uuid(ulid: Ulid) -> String {
        let bytes = ulid.to_bytes();
        // Format as 8-4-4-4-12 canonical UUID with dashes.
        format!(
            "{:02x}{:02x}{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
            bytes[0],
            bytes[1],
            bytes[2],
            bytes[3],
            bytes[4],
            bytes[5],
            bytes[6],
            bytes[7],
            bytes[8],
            bytes[9],
            bytes[10],
            bytes[11],
            bytes[12],
            bytes[13],
            bytes[14],
            bytes[15]
        )
    }

    const KNOWN_BYTES: [u8; 16] = [
        0x01, 0x67, 0x4f, 0x3a, 0x4e, 0x32, 0xdf, 0x41, 0x8c, 0x6b, 0x27, 0x4e, 0x73, 0x91, 0x5f,
        0xce,
    ];

    #[test]
    fn uuid_to_ulid_decodes_base64_url_no_pad_correctly() {
        let expected = Ulid::from_bytes(KNOWN_BYTES);
        let b64 = encode_ulid_as_base64_url_no_pad(expected);

        let decoded = uuid_to_ulid(&b64);

        assert_eq!(decoded, expected);
    }

    #[test]
    fn uuid_to_ulid_decodes_base64_url_padded_correctly() {
        let expected = Ulid::from_bytes(KNOWN_BYTES);
        let b64 = encode_ulid_as_base64_url_padded(expected);

        let decoded = uuid_to_ulid(&b64);

        assert_eq!(decoded, expected);
    }

    #[test]
    fn uuid_to_ulid_decodes_hex_uuid_with_dashes_correctly() {
        let expected = Ulid::from_bytes(KNOWN_BYTES);
        let hex = encode_ulid_as_hex_uuid(expected);

        let decoded = uuid_to_ulid(&hex);

        assert_eq!(decoded, expected);
    }

    #[test]
    fn uuid_to_ulid_roundtrip_for_random_ulid() {
        let original = Ulid::new();
        let b64 = encode_ulid_as_base64_url_no_pad(original);

        let decoded = uuid_to_ulid(&b64);

        assert_eq!(decoded, original);
    }

    #[test]
    fn uuid_to_ulid_returns_nil_for_invalid_input() {
        assert_eq!(uuid_to_ulid("not-a-valid-base64-uuid"), Ulid::nil());
    }

    #[test]
    fn uuid_to_ulid_returns_nil_for_empty_input() {
        assert_eq!(uuid_to_ulid(""), Ulid::nil());
    }

    #[test]
    fn uuid_to_ulid_returns_nil_for_truncated_base64() {
        assert_eq!(uuid_to_ulid("AQEBAQEBAQE"), Ulid::nil());
    }

    #[test]
    fn uuid_to_ulid_returns_nil_for_hex_wrong_length() {
        assert_eq!(uuid_to_ulid("550e8400-e29b-41d4"), Ulid::nil());
    }

    #[test]
    fn redact_hides_middle_of_long_id() {
        let redacted = redact("abcdefgh1234567890");
        assert!(redacted.contains('…'));
        assert!(!redacted.contains("efgh1234"));
    }

    #[test]
    fn redact_masks_short_input() {
        assert_eq!(redact("abc"), "***");
    }
}
