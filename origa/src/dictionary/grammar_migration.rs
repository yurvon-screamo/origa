//! Grammar corpus v3 card migration (content pass, #503 pipeline).
//!
//! The v3 corpus hard-deletes 43 rules (pseudo-points, form-category
//! umbrellas, lexicon, merge twins — see `scripts/apply_pruning_staging.py`).
//! Released clients carry user FSRS cards keyed by `rule_id`; loading a
//! corpus without those rules must not leave broken cards behind.
//!
//! Merge twins (5 pairs) have a successor: the surviving rule absorbed the
//! twin's detector anchors and related patterns, so the card's study
//! history transfers. Deleted rules without a successor are lexicon or
//! lesson-phrase points — their cards are dropped: there is nothing left
//! to review.
//!
//! The map is generated from `staging/migration_map.json`; `rule_id`s are
//! stable ULIDs, so the pairs are compile-time constants.

use ulid::Ulid;

/// (deleted twin rule_id, surviving successor rule_id) — merge pairs only.
/// Public for the corpus-consistency invariant test: one source of truth.
pub const MERGE_TWINS: [(&str, &str); 5] = [
    ("01KV2C1TJN7FZ34PB80VBFCXFD", "01KV2BRAW30ESEMGXK3N2PTAF4"),
    ("01KV2BV4G2TVKG953YH43902ZH", "01KV2C1TJN7FZ34PB80VBFCXFM"),
    ("01KV2C1TJN7FZ34PB80VBFCXFE", "01KV2BRAW30ESEMGXK3N2PTAF5"),
    ("01KV2BV4G2TVKG953YH43902ZK", "01KV2C1TJN7FZ34PB80VBFCXEC"),
    ("01KV2BV49PQW2NZ6JX06ZRX0FB", "01KV2BRAW30ESEMGXK3N2PTAEA"),
];

/// Successor for a merge-deleted rule, if any. Rules deleted without a
/// successor return `None` and their cards are dropped.
pub fn migration_successor(rule_id: &Ulid) -> Option<Ulid> {
    let text = rule_id.to_string();
    MERGE_TWINS
        .iter()
        .find(|(deleted, _)| *deleted == text)
        .and_then(|(_, successor)| Ulid::from_string(successor).ok())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn twin_resolves_to_its_successor() {
        let deleted = Ulid::from_string("01KV2C1TJN7FZ34PB80VBFCXFD").unwrap();
        let successor = migration_successor(&deleted).expect("twin must resolve");
        assert_eq!(successor.to_string(), "01KV2BRAW30ESEMGXK3N2PTAF4");
    }

    #[test]
    fn unrelated_rule_has_no_successor() {
        assert!(migration_successor(&Ulid::new()).is_none());
    }

    #[test]
    fn all_successors_are_valid_ulids() {
        for (deleted, successor) in MERGE_TWINS {
            assert!(
                Ulid::from_string(deleted).is_ok(),
                "bad deleted id {deleted}"
            );
            assert!(
                Ulid::from_string(successor).is_ok(),
                "bad successor {successor}"
            );
        }
    }
}
