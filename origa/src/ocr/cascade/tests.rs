use super::*;
use rstest::rstest;
use std::io::Write;
use tempfile::NamedTempFile;

struct FakeLineRecognizer {
    text: String,
}

impl LineRecognizer for FakeLineRecognizer {
    fn read_line(&self, _image: &DynamicImage) -> String {
        self.text.clone()
    }
}

fn cascade_with(narrow: &str, mid: &str, wide: &str) -> CascadeRecognizer<FakeLineRecognizer> {
    CascadeRecognizer::from_recognizers(
        FakeLineRecognizer {
            text: narrow.to_string(),
        },
        FakeLineRecognizer {
            text: mid.to_string(),
        },
        FakeLineRecognizer {
            text: wide.to_string(),
        },
    )
}

fn img() -> DynamicImage {
    DynamicImage::new_rgb8(1, 1)
}

#[rstest]
#[case::one_point_nine_below_both_tiers(1.9, "none")]
#[case::just_below_two(1.95, "mid")]
#[case::exact_two(2.0, "mid")]
#[case::just_above_two(2.05, "mid")]
#[case::still_within_two(2.1, "mid")]
#[case::between_tiers(2.5, "none")]
#[case::below_narrow_zone(2.85, "none")]
#[case::f32_nine_tenths_inside_three(2.9, "narrow")]
#[case::just_below_three(2.95, "narrow")]
#[case::exact_three(3.0, "narrow")]
#[case::just_above_three(3.05, "narrow")]
#[case::f32_ten_tenths_inside_three(3.1, "narrow")]
fn approx_eq_matches_tier_targets_within_f32_epsilon(#[case] value: f32, #[case] tier: &str) {
    // f32 reality check: `approx_eq` compares in f32, so 2.9 and 3.1 land
    // INSIDE the epsilon zone of 3.0 (|3.1f32 - 3.0| = 0.09999990 < 0.1f32),
    // while 1.9 lands OUTSIDE the zone of 2.0 (0.10000002 > 0.1f32). The
    // matrix pins the actual f32 semantics, not decimal intuition.
    assert_eq!(approx_eq(value, 3.0), tier == "narrow", "value: {value}");
    assert_eq!(approx_eq(value, 2.0), tier == "mid", "value: {value}");
}

#[rstest]
#[case::three_routes_narrow(3.0, "narrow")]
#[case::two_routes_mid(2.0, "mid")]
#[case::f32_close_to_three_still_routes_narrow(2.9, "narrow")]
#[case::f32_three_point_one_still_routes_narrow(3.1, "narrow")]
#[case::gap_below_two_falls_through_to_wide(1.9, "wide")]
#[case::midpoint_falls_through_to_wide(2.5, "wide")]
fn recognize_routes_by_predicted_char_count(#[case] pred: f32, #[case] tier_text: &str) {
    // Short readings never escalate: the returned text names the tier.
    let cascade = cascade_with("narrow", "mid", "wide");

    // Act
    let result = cascade.recognize(&img(), pred);

    // Assert
    assert_eq!(result, tier_text, "pred: {pred}");
}

// `text.len()` in the cascade counts UTF-8 bytes; ASCII strings control
// byte length exactly.
#[rstest]
#[case::below_threshold_keeps_narrow((24, 45), "a".repeat(24))]
#[case::at_threshold_keeps_short_mid((25, 44), "b".repeat(44))]
#[case::at_threshold_escalates_long_mid_to_wide((25, 45), "c".repeat(45))]
fn recognize_escalates_from_narrow_by_byte_length(
    #[case] lens: (usize, usize),
    #[case] expected: String,
) {
    // Arrange
    let (narrow_len, mid_len) = lens;
    let narrow = "a".repeat(narrow_len);
    let mid = "b".repeat(mid_len);
    let cascade = cascade_with(&narrow, &mid, &"c".repeat(45));

    // Act
    let result = cascade.recognize(&img(), 3.0);

    // Assert
    assert_eq!(result, expected);
}

#[rstest]
#[case::short_mid_reading_stays(44, "b".repeat(44))]
#[case::long_mid_reading_escalates_to_wide(45, "c".repeat(45))]
fn recognize_from_mid_tier_start_escalates_by_byte_length(
    #[case] mid_len: usize,
    #[case] expected: String,
) {
    // Arrange
    let mid = "b".repeat(mid_len);
    let cascade = cascade_with("narrow", &mid, &"c".repeat(45));

    // Act
    let result = cascade.recognize(&img(), 2.0);

    // Assert
    assert_eq!(result, expected);
}

#[test]
fn new_with_missing_vocab_file_errors() {
    // Arrange
    let missing = std::env::temp_dir().join("origa-no-such-cascade-vocab.txt");

    // Act
    let result = CascadeRecognizer::new(&missing, &missing, &missing, &missing);

    // Assert
    let Err(OrigaError::OcrError { reason }) = result else {
        panic!("expected a vocabulary-loading failure");
    };
    assert!(reason.contains("vocabulary"), "reason: {reason}");
}

#[test]
fn new_with_valid_vocab_but_missing_model_errors() {
    // Arrange
    let mut vocab_file = NamedTempFile::new().expect("temp vocab file");
    writeln!(vocab_file, "あ").expect("vocab line");
    let missing = std::env::temp_dir().join("origa-no-such-parseq.onnx");

    // Act
    let result = CascadeRecognizer::new(&missing, &missing, &missing, vocab_file.path());

    // Assert
    let Err(OrigaError::OcrError { reason }) = result else {
        panic!("expected a model-loading failure");
    };
    assert!(
        reason.contains("Failed to load PARSeq model"),
        "reason: {reason}"
    );
}
