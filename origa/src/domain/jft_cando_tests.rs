use std::collections::HashSet;
use std::sync::LazyLock;

use rstest::rstest;

use super::{JftBand, JftCandoCatalog};
use crate::domain::OrigaError;

/// Byte-identical twin of the deployed `well_known_set/irodori_cando.json`,
/// emitted by the same `scripts/generate_irodori_cando.py` run. The totals
/// assertions below fail loudly if the generator output ever drifts.
const FIXTURE: &str = include_str!("../../tests/fixtures/irodori_cando.json");

/// Band totals of the official dataset (341 Can-dos: nyuumon 105 + 2
/// classroom, shokyuu1 109, shokyuu2 125), in learning-path order.
const EXPECTED_TOTALS: [(JftBand, usize); 3] =
    [(JftBand::A1, 107), (JftBand::A21, 109), (JftBand::A2, 125)];

static CATALOG: LazyLock<JftCandoCatalog> =
    LazyLock::new(|| JftCandoCatalog::parse(FIXTURE).expect("fixture must parse"));

/// Distinct lessons per band, re-derived from the fixture JSON so the test
/// keeps a single source of truth (the generator output itself).
fn fixture_lessons_of_band(band: JftBand) -> HashSet<String> {
    let document: serde_json::Value = serde_json::from_str(FIXTURE).expect("fixture is valid JSON");
    document["can_dos"]
        .as_array()
        .expect("can_dos array")
        .iter()
        .filter(|entry| entry["band"].as_str() == Some(band.code()))
        .map(|entry| entry["lesson"].as_str().expect("lesson str").to_string())
        .collect()
}

#[test]
fn parse_reads_fixture_with_approved_band_totals() {
    // Act
    let coverage = CATALOG.coverage(&HashSet::new());

    // Assert: every band present, exact official totals, nothing covered.
    for (band, total) in EXPECTED_TOTALS {
        let band_coverage = coverage
            .bands()
            .iter()
            .find(|c| c.band == band)
            .unwrap_or_else(|| panic!("band {} missing", band.code()));
        assert_eq!(band_coverage.total, total, "band {}", band.code());
        assert_eq!(band_coverage.covered, 0, "no imports yet");
        assert_eq!(band_coverage.percentage(), 0.0);
    }
}

#[rstest]
#[case::a1(JftBand::A1, "irodori_nyuumon_01")]
#[case::a2_1(JftBand::A21, "irodori_shokyuu1_01")]
#[case::a2(JftBand::A2, "irodori_shokyuu2_01")]
fn importing_one_lesson_moves_only_its_own_band(#[case] band: JftBand, #[case] lesson: &str) {
    // Arrange
    let imported: HashSet<String> = [lesson.to_string()].into_iter().collect();

    // Act
    let coverage = CATALOG.coverage(&imported);

    // Assert: the touched band is strictly positive, the others untouched.
    for band_coverage in coverage.bands() {
        if band_coverage.band == band {
            assert!(
                band_coverage.covered > 0,
                "lesson {} must carry Can-dos",
                lesson
            );
        } else {
            assert_eq!(
                band_coverage.covered,
                0,
                "band {} must stay untouched",
                band_coverage.band.code()
            );
        }
    }
}

#[test]
fn importing_every_lesson_yields_full_coverage() {
    // Arrange: every lesson of every band.
    let all_lessons: HashSet<String> = JftBand::ALL
        .into_iter()
        .flat_map(fixture_lessons_of_band)
        .collect();

    // Act
    let coverage = CATALOG.coverage(&all_lessons);

    // Assert: 100% everywhere, all 341 Can-dos covered.
    for band_coverage in coverage.bands() {
        assert_eq!(
            band_coverage.percentage(),
            100.0,
            "band {}",
            band_coverage.band.code()
        );
    }
    assert_eq!(coverage.total_covered(), 341);
}

#[test]
fn partial_import_scales_bands_independently() {
    // Arrange: complete Shokyuu 1, nothing else.
    let imported = fixture_lessons_of_band(JftBand::A21);

    // Act
    let coverage = CATALOG.coverage(&imported);

    // Assert
    let bands = coverage.bands();
    assert_eq!(bands[0].band, JftBand::A1);
    assert_eq!(bands[0].percentage(), 0.0);
    assert_eq!(bands[1].band, JftBand::A21);
    assert_eq!(bands[1].percentage(), 100.0);
    assert_eq!(bands[2].band, JftBand::A2);
    assert_eq!(bands[2].percentage(), 0.0);
    assert_eq!(coverage.total_covered(), 109);
}

#[test]
fn bands_are_ordered_along_the_learning_path() {
    let coverage = CATALOG.coverage(&HashSet::new());
    let codes: Vec<&str> = coverage.bands().iter().map(|b| b.band.code()).collect();
    assert_eq!(codes, vec!["A1", "A2.1", "A2"]);
}

#[test]
fn current_band_is_a1_while_its_coverage_is_below_the_threshold() {
    // Arrange: one Nyuumon lesson — A1 partially covered.
    let imported: HashSet<String> = ["irodori_nyuumon_01".to_string()].into_iter().collect();

    // Act
    let current = CATALOG.coverage(&imported).current().clone();

    // Assert
    assert_eq!(current.band, JftBand::A1);
    assert!(current.percentage() < 90.0);
}

#[test]
fn current_band_advances_only_past_the_cleared_predecessor() {
    // Arrange: the onboarding flow imports cumulatively, so completing
    // Shokyuu 1 means all of Nyuumon plus Shokyuu 1 — A1 and A2.1 fully
    // covered, and the learner is working through A2 while it is empty.
    let imported: HashSet<String> = [JftBand::A1, JftBand::A21]
        .into_iter()
        .flat_map(fixture_lessons_of_band)
        .collect();

    // Act
    let coverage = CATALOG.coverage(&imported);
    let current = coverage.current();

    // Assert
    assert_eq!(current.band, JftBand::A2);
    assert_eq!(current.percentage(), 0.0);
}

#[test]
fn current_band_stays_on_a2_when_everything_is_cleared() {
    // Arrange: every lesson of every band.
    let all_lessons: HashSet<String> = JftBand::ALL
        .into_iter()
        .flat_map(fixture_lessons_of_band)
        .collect();

    // Act
    let current = CATALOG.coverage(&all_lessons).current().clone();

    // Assert: the tracker parks on the pass level at its real percentage.
    assert_eq!(current.band, JftBand::A2);
    assert_eq!(current.percentage(), 100.0);
}

#[rstest]
#[case::empty_object("{}")]
#[case::missing_array(r#"{"version": 1}"#)]
#[case::not_json("not json at all")]
fn parse_rejects_malformed_documents(#[case] json: &str) {
    let result = JftCandoCatalog::parse(json);

    assert!(matches!(
        result,
        Err(OrigaError::WellKnownSetParseError { .. })
    ));
}

#[test]
fn parse_rejects_unknown_band_and_level_tags() {
    // The enums are the data contract: unknown official tags must fail loudly
    // instead of slipping through as opaque strings.
    // Arrange
    let bad_band =
        r#"{"can_dos": [{"id": 1, "level": "A1", "band": "B1", "lesson": "irodori_nyuumon_01"}]}"#;
    let bad_level =
        r#"{"can_dos": [{"id": 1, "level": "C1", "band": "A1", "lesson": "irodori_nyuumon_01"}]}"#;

    // Act
    let band_result = JftCandoCatalog::parse(bad_band);
    let level_result = JftCandoCatalog::parse(bad_level);

    // Assert
    assert!(matches!(
        band_result,
        Err(OrigaError::WellKnownSetParseError { .. })
    ));
    assert!(matches!(
        level_result,
        Err(OrigaError::WellKnownSetParseError { .. })
    ));
}

#[test]
fn fixture_carries_both_official_level_tags() {
    // Parse-time validation already rejects foreign tags; this pins that the
    // real dataset exercises both official values (no silent narrowing).
    // Reads through serde_json so the check survives generator formatting
    // changes (indent, key order).
    let document: serde_json::Value = serde_json::from_str(FIXTURE).expect("fixture is valid JSON");
    let levels: HashSet<&str> = document["can_dos"]
        .as_array()
        .expect("can_dos array")
        .iter()
        .map(|entry| entry["level"].as_str().expect("level str"))
        .collect();
    assert!(
        levels.contains("A1") && levels.contains("A2"),
        "levels: {levels:?}"
    );
}
