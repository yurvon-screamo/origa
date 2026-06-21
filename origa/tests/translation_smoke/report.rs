//! TSV + summary printer for ``ProblemReport`` collections.
//!
//! Output format (spaces instead of tabs in this doc-comment to satisfy
//! clippy::tabs_in_doc_comments):
//!
//! ```text
//! === TSV ===
//! kind    phrase_idx    token_idx    pos    surface    base    detail    phrase_text
//! F5      300           0            AuxiliaryVerb    ム    ム    aux-like katakana ...
//! ...
//!
//! === SUMMARY ===
//! total problems: 1234
//! F1    FAIL    0 (TokenizeError)
//! ...
//! ```

use super::harness::{ProblemKind, ProblemReport};

const SUMMARY_ORDER: &[ProblemKind] = &[
    ProblemKind::TokenizeError,
    ProblemKind::MissingGrammarLabelKnownAux,
    ProblemKind::EmptyBaseForm,
    ProblemKind::SuspiciousKatakanaBase,
    ProblemKind::UnknownWordNoTranslation,
    ProblemKind::ConjugatedNoGrammarLabel,
    ProblemKind::AuxiliaryNoGrammarLabel,
    ProblemKind::IncompleteDictionaryEntry,
];

pub fn print_report(reports: &[ProblemReport]) {
    print_tsv(reports);
    print_summary(reports);
}

fn print_tsv(reports: &[ProblemReport]) {
    eprintln!();
    eprintln!("kind\tphrase_idx\ttoken_idx\tpos\tsurface\tbase\tdetail\tphrase_text");
    for r in reports {
        eprintln!(
            "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
            r.kind.as_code(),
            r.phrase_idx,
            r.token_idx,
            r.pos,
            r.surface,
            r.base,
            r.detail,
            r.phrase_text.chars().take(60).collect::<String>()
        );
    }
}

fn print_summary(reports: &[ProblemReport]) {
    eprintln!();
    eprintln!("=== SUMMARY ===");
    eprintln!("total problems: {}", reports.len());

    for kind in SUMMARY_ORDER {
        let matching: Vec<&ProblemReport> = reports.iter().filter(|r| r.kind == *kind).collect();
        let fail_marker = if kind.is_fail() { "FAIL" } else { "WARN" };
        eprintln!(
            "{}\t{}\t{} ({:?})",
            kind.as_code(),
            fail_marker,
            matching.len(),
            kind
        );
        for r in matching.iter().take(5) {
            eprintln!(
                "    [phrase#{} tok#{}] 「{}」 (base 「{}」) — {}",
                r.phrase_idx, r.token_idx, r.surface, r.base, r.detail
            );
        }
    }
}
