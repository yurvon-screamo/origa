//! Corpus coverage for grammar quiz distractors (#503).
//!
//! Walks every quiz-capable chain in `cdn/grammar/grammar_v3.json` and
//! asserts that [`origa::domain::generate_grammar_distractors`] fields
//! three near-miss options. The only rules allowed to lose quizzes are
//! the pinned na-adjective pair, whose kanji stems have no kana to break.
//!
//! The `cdn/` directory is gitignored; on a fresh clone without the
//! grammar store the test **gracefully skips** (pass with a stderr note)
//! — same convention as `grammar_regression_checks.rs`. The pinned
//! coverage numbers are therefore only truly enforced in environments
//! that have the CDN artifacts (local dev, release CI).
//!
//! Run: `cargo test -p origa --test near_miss_coverage -- --nocapture`.

use std::path::PathBuf;
use std::sync::Once;

use origa::dictionary::grammar::{
    FormatActionGroup, GrammarData, init_grammar, is_grammar_loaded, iter_grammar_rules,
};
use origa::domain::{PartOfSpeech, apply_format_actions, generate_grammar_distractors};

static CORPUS_INIT: Once = Once::new();

fn ensure_corpus_loaded() -> bool {
    let mut loaded = false;
    CORPUS_INIT.call_once(|| {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("workspace root is parent of the origa crate manifest")
            .join("cdn")
            .join("grammar")
            .join("grammar_v3.json");
        let Ok(grammar_json) = std::fs::read_to_string(&path) else {
            return;
        };
        init_grammar(GrammarData { grammar_json }).expect("grammar store must initialize");
        loaded = is_grammar_loaded();
    });
    loaded || is_grammar_loaded()
}

const SAMPLE_WORDS: &[(PartOfSpeech, &[&str])] = &[
    (
        PartOfSpeech::Verb,
        &["書く", "食べる", "買う", "する", "来る"],
    ),
    (PartOfSpeech::IAdjective, &["高い"]),
    (PartOfSpeech::NaAdjective, &["静か"]),
];

/// Pinned by plan review: the only rules allowed to lose quizzes. Both
/// are pure na-adjective chains (kanji stem, no kana to mutate).
const PINNED_LOSSES: &[&str] = &[
    "01G00000000000000060000000", // ～な＋名詞 [AdjectiveToNa]
    "01G000000000000000SW000000", // ～て／～くて／～で [AdjectiveToDe]
];

/// Pinned: chains that reject every typical sample word (lemma-gated
/// honorific replacements) — they quiz only when the user's vocabulary
/// contains the specific lemma, so typical-word coverage is vacuous.
const PINNED_INAPPLICABLE: &[&str] = &[
    "01KXAX0VRP192A16Z4NJ2WK9GA", // いらっしゃる→いらっしゃい [VerbToIrasshai]
    "01KXAX0VRPNSR58CP87HJPXYMM", // 為さる→なさい [VerbToNasai]
    "01KXAX0VRPXYAP6KYN9P7CJTPA", // 下さる→ください [VerbToKudasai]
];

#[test]
fn corpus_quiz_chains_coverage_matches_pinned_losses() {
    if !ensure_corpus_loaded() {
        eprintln!("grammar store absent — corpus coverage check skipped");
        return;
    }

    let mut losing: Vec<String> = Vec::new();
    let mut inapplicable: Vec<String> = Vec::new();
    let mut checked = 0usize;
    for rule in iter_grammar_rules() {
        if !rule.has_format_map() {
            continue;
        }
        for (pos, words) in SAMPLE_WORDS {
            let Some(actions) = rule.format_actions_for_pos(pos) else {
                continue;
            };
            checked += 1;
            // Strict coverage: only words the chain actually applies to
            // count. A rule "loses" when no applicable sample word fields
            // three distractors; a rule with no applicable sample word at
            // all (lemma-gated honorifics like 下さる→ください) cannot be
            // exercised by typical vocabulary and is pinned separately.
            let mut applicable = 0usize;
            let covered = words.iter().any(|word| {
                let Ok(correct) = apply_format_actions(word, actions, pos) else {
                    return false;
                };
                applicable += 1;
                let distractors = generate_grammar_distractors(actions, word, pos, &correct, 3);
                if distractors.len() < 3 {
                    eprintln!(
                        "LOSS {} word={word} correct={correct} distractors={distractors:?}",
                        rule.rule_id(),
                    );
                }
                distractors.len() >= 3
            });
            if applicable == 0 {
                inapplicable.push(rule.rule_id().to_string());
            } else if !covered {
                losing.push(rule.rule_id().to_string());
            }
        }
    }

    losing.sort();
    inapplicable.sort();
    let pinned = PINNED_LOSSES
        .iter()
        .map(|s| s.to_string())
        .collect::<Vec<_>>();
    let pinned_inapplicable = PINNED_INAPPLICABLE
        .iter()
        .map(|s| s.to_string())
        .collect::<Vec<_>>();
    assert_eq!(
        losing, pinned,
        "quiz chains losing quizzes must match the pinned list ({checked} chains checked)"
    );
    assert_eq!(
        inapplicable, pinned_inapplicable,
        "chains with no applicable sample word must match the pinned list"
    );
}

/// Every distractor across the whole corpus must keep the correct
/// answer's visible rule identity: it either shares a common prefix with
/// the correct answer or shares its tail. This is the structural guard
/// against the original bug — options from unrelated rules.
#[test]
fn corpus_distractors_stay_within_the_rule_shape() {
    if !ensure_corpus_loaded() {
        eprintln!("grammar store absent — shape check skipped");
        return;
    }

    let mut checked_pairs = 0usize;
    for rule in iter_grammar_rules() {
        if !rule.has_format_map() {
            continue;
        }
        for (pos, words) in SAMPLE_WORDS {
            let Some(actions) = rule.format_actions_for_pos(pos) else {
                continue;
            };
            let has_postfix_tail = actions
                .iter()
                .any(|a| a.group() == FormatActionGroup::Universal);

            for word in words.iter() {
                let Ok(correct) = apply_format_actions(word, actions, pos) else {
                    continue;
                };
                for distractor in generate_grammar_distractors(actions, word, pos, &correct, 3) {
                    checked_pairs += 1;
                    // Near-miss tripwire: the distractor must stay
                    // structurally anchored to the correct answer —
                    // sharing its head (stem survives: 書いだ/書いた,
                    // 書か/書け) or its tail (ending/tail survives:
                    // するます/します, するて/して, するか/しませんか).
                    // The precise behavioral contracts are pinned by the
                    // unit tests in `quiz_generation`; this walk guards
                    // the whole corpus against unrelated-rule garbage.
                    let prefix = correct
                        .chars()
                        .zip(distractor.chars())
                        .take_while(|(c, d)| c == d)
                        .count();
                    let suffix = correct
                        .chars()
                        .rev()
                        .zip(distractor.chars().rev())
                        .take_while(|(c, d)| c == d)
                        .count();
                    let shares_shape = if has_postfix_tail {
                        suffix >= 1
                    } else {
                        prefix >= 1 || suffix >= 1
                    };
                    assert!(
                        shares_shape,
                        "distractor {distractor} does not share the rule shape of {correct} \
                         (rule {})",
                        rule.rule_id(),
                    );
                }
            }
        }
    }
    assert!(checked_pairs > 100, "expected a real corpus walkthrough");
}

/// Human-auditable dump of the full corpus quiz option sets (#503):
/// `cargo test -p origa --test near_miss_coverage -- --ignored dump_quadruples --nocapture`.
/// Pedagogical quality (plausible vs silly breaks) is a human judgment —
/// this prints every generated option set for eyeballing.
#[test]
#[ignore = "manual audit dump, not a pass/fail check"]
fn dump_quadruples() {
    if !ensure_corpus_loaded() {
        eprintln!("grammar store absent — dump skipped");
        return;
    }

    for rule in iter_grammar_rules() {
        if !rule.has_format_map() {
            continue;
        }
        for (pos, words) in SAMPLE_WORDS {
            let Some(actions) = rule.format_actions_for_pos(pos) else {
                continue;
            };
            for word in words.iter() {
                let Ok(correct) = apply_format_actions(word, actions, pos) else {
                    continue;
                };
                let distractors = generate_grammar_distractors(actions, word, pos, &correct, 3);
                println!(
                    "{} [{pos:?}] {word} → {correct} ✅ | {}",
                    rule.rule_id(),
                    distractors.join(" ❌ "),
                );
            }
        }
    }
}
