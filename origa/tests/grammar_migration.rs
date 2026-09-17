//! Integration tests for the v3 grammar card migration (#503 content pass).
//!
//! Lives in `tests/` (own process): the lib-crate grammar OnceLock stays
//! owned by the production-corpus tests, while here a synthetic corpus
//! loads freely. Cards referencing rules that the corpus no longer has
//! cannot even be created through `User::create_card` (it validates the
//! rule), so the fixtures insert ghost cards directly into the study map —
//! exactly the state released clients end up in after the corpus swap.

use origa::dictionary::grammar::{GrammarData, init_grammar, is_grammar_loaded};
use origa::domain::{Card, GrammarRuleCard, NativeLanguage, User};
use origa::use_cases::migrate_user_cards;
use std::sync::Once;
use ulid::Ulid;

/// Survivor of the ～される merge + an unrelated live rule + the successor
/// of the ～たら、～た pair (for the duplicate-transfer case).
const LIVE_RULE: &str = "01KV2BRAW30ESEMGXK3N2PTAF4"; // ～される twin survivor
const TWIN_OF_LIVE: &str = "01KV2C1TJN7FZ34PB80VBFCXFD"; // deleted twin
const OTHER_SUCCESSOR: &str = "01KV2BRAW30ESEMGXK3N2PTAEA"; // ～たら、～た survivor
const OTHER_TWIN: &str = "01KV2BV49PQW2NZ6JX06ZRX0FB"; // its deleted twin

static CORPUS_INIT: Once = Once::new();

fn ensure_corpus() {
    CORPUS_INIT.call_once(|| {
        let json = serde_json::json!({
            "grammar": [
                {"rule_id": LIVE_RULE, "level": "N3",
                 "content": {"English": {"title": "t1", "short_description": "s",
                 "explanation": "e", "how_to_form": "f", "examples": "[]",
                 "nuances": {}, "pro_tip": ""}}},
                {"rule_id": OTHER_SUCCESSOR, "level": "N3",
                 "content": {"English": {"title": "t2", "short_description": "s",
                 "explanation": "e", "how_to_form": "f", "examples": "[]",
                 "nuances": {}, "pro_tip": ""}}}
            ]
        })
        .to_string();
        init_grammar(GrammarData { grammar_json: json }).expect("synthetic corpus must load");
    });
    assert!(is_grammar_loaded(), "corpus must be loaded");
}

fn user() -> User {
    User::new(
        "migr@example.com".to_string(),
        NativeLanguage::English,
        None,
    )
}

/// Insert a card that references a rule the loaded corpus does NOT have —
/// the ghost state of a client that studied a pre-v3 corpus. `create_card`
/// validates the rule, so the ghost is produced by a serde round-trip:
/// create a live card, then rewrite its rule_id inside the serialized
/// knowledge set.
fn insert_ghost_card(user: &mut User, ghost_rule: &str) -> Ulid {
    let live = user
        .create_card(Card::Grammar(
            GrammarRuleCard::new(Ulid::from_string(LIVE_RULE).expect("live ulid"))
                .expect("grammar card struct"),
        ))
        .expect("live card");
    let card_id = *live.card_id();

    let mut value = serde_json::to_value(&*user).expect("user serializes");
    let cards = value
        .get_mut("knowledge_set")
        .and_then(|v| v.get_mut("study_cards"))
        .expect("study_cards in serialized user");
    if let Some(map) = cards.as_object_mut() {
        if let Some(card) = map.get_mut(&card_id.to_string()) {
            if let Some(grammar) = card.get_mut("card").and_then(|c| c.get_mut("Grammar")) {
                grammar["rule_id"] = serde_json::json!(ghost_rule);
            }
        }
    }
    *user = serde_json::from_value(value).expect("user deserializes");
    card_id
}

fn insert_live_card(user: &mut User, rule_id: &str) -> Ulid {
    let rid = Ulid::from_string(rule_id).expect("valid ulid");
    let card = user
        .create_card(Card::Grammar(
            GrammarRuleCard::new(rid).expect("grammar card struct"),
        ))
        .expect("live rule card");
    *card.card_id()
}

fn grammar_rule_ids(user: &User) -> Vec<String> {
    let mut ids: Vec<String> = user
        .knowledge_set()
        .study_cards()
        .values()
        .filter_map(|s| match s.card() {
            Card::Grammar(grc) => Some(grc.rule_id().to_string()),
            _ => None,
        })
        .collect();
    ids.sort();
    ids
}

#[test]
fn twin_ghost_transfers_onto_surviving_rule() {
    ensure_corpus();

    // Arrange
    let mut user = user();
    let ghost = insert_ghost_card(&mut user, TWIN_OF_LIVE);
    let _ = ghost;

    // Act
    let report = migrate_user_cards(&mut user).expect("migration");

    // Assert
    assert_eq!(report.transferred, 1);
    assert_eq!(report.dropped_deleted, 0);
    assert_eq!(grammar_rule_ids(&user), vec![LIVE_RULE.to_string()]);
}

#[test]
fn twin_ghost_with_native_successor_card_drops_duplicate() {
    ensure_corpus();

    // Arrange: the user already studies the survivor AND holds the twin ghost.
    let mut user = user();
    let native = insert_live_card(&mut user, OTHER_SUCCESSOR);
    let ghost = insert_ghost_card(&mut user, OTHER_TWIN);
    assert_ne!(native, ghost);

    // Act
    let report = migrate_user_cards(&mut user).expect("migration");

    // Assert: the ghost is dropped, the native survivor card keeps its slot.
    assert_eq!(report.transferred, 0);
    assert_eq!(report.dropped_duplicates, 1);
    assert_eq!(grammar_rule_ids(&user), vec![OTHER_SUCCESSOR.to_string()]);
    assert!(user.knowledge_set().get_card(native).is_some());
    assert!(user.knowledge_set().get_card(ghost).is_none());
}

#[test]
fn deleted_rule_ghost_drops_without_successor() {
    ensure_corpus();

    // Arrange: a rule with no migration successor (lexicon deletion).
    let mut user = user();
    insert_ghost_card(&mut user, "01G0000000000000000C000000"); // ～も N5 D-deletion

    // Act
    let report = migrate_user_cards(&mut user).expect("migration");

    // Assert
    assert_eq!(report.dropped_deleted, 1);
    assert!(grammar_rule_ids(&user).is_empty());
}

#[test]
fn second_pass_is_a_noop() {
    ensure_corpus();

    // Arrange
    let mut user = user();
    insert_ghost_card(&mut user, TWIN_OF_LIVE);
    migrate_user_cards(&mut user).expect("first pass");

    // Act
    let report = migrate_user_cards(&mut user).expect("second pass");

    // Assert
    assert_eq!(report.total(), 0);
    assert_eq!(grammar_rule_ids(&user), vec![LIVE_RULE.to_string()]);
}

/// The merge map must stay consistent with the SHIPPED corpus: every
/// successor alive, every deleted twin gone. A future corpus cleanup that
/// drops a successor would silently break the migration (transfers turn
/// into endless errors) — this test fails loudly instead. Graceful-skips
/// when the gitignored v3 corpus is absent (fresh clone / CI cache miss).
#[test]
fn merge_twins_match_the_shipped_v3_corpus() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("cdn/grammar/grammar_v3.json");
    let Ok(raw) = std::fs::read_to_string(&path) else {
        eprintln!("grammar_v3.json not present — skipping");
        return;
    };
    let corpus: serde_json::Value = serde_json::from_str(&raw).expect("v3 corpus parses");
    let live: std::collections::HashSet<String> = corpus["grammar"]
        .as_array()
        .expect("grammar array")
        .iter()
        .map(|r| r["rule_id"].as_str().expect("rule_id").to_string())
        .collect();

    let twins: &[(&str, &str)] = &[
        ("01KV2C1TJN7FZ34PB80VBFCXFD", "01KV2BRAW30ESEMGXK3N2PTAF4"),
        ("01KV2BV4G2TVKG953YH43902ZH", "01KV2C1TJN7FZ34PB80VBFCXFM"),
        ("01KV2C1TJN7FZ34PB80VBFCXFE", "01KV2BRAW30ESEMGXK3N2PTAF5"),
        ("01KV2BV4G2TVKG953YH43902ZK", "01KV2C1TJN7FZ34PB80VBFCXEC"),
        ("01KV2BV49PQW2NZ6JX06ZRX0FB", "01KV2BRAW30ESEMGXK3N2PTAEA"),
    ];
    for (deleted, successor) in twins {
        assert!(
            !live.contains(*deleted),
            "twin {deleted} must be absent from the v3 corpus"
        );
        assert!(
            live.contains(*successor),
            "successor {successor} must stay alive in the v3 corpus — \
             the migration map (dictionary/grammar_migration.rs) is stale"
        );
    }
}
