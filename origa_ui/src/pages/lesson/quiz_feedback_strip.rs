use crate::i18n::*;
use crate::ui_components::{MarkdownText, MarkdownVariant};
use leptos::prelude::*;
use std::collections::HashSet;

use super::quiz_ledger::{LedgerRow, LedgerRowKind, LedgerVerdict};
use super::quiz_ledger_labels::{row_label, verdict_label};

/// The quiz feedback strip — the structured answer ledger below the
/// lesson card (task 3SekkMiYYW, direction B): an aged-paper band with
/// a verdict-colored left accent, a verdict line and ruled answer
/// rows. Replaces the old in-card «verdict + missed + extra» text
/// flow; the reading order is always question → feedback → action.
///
/// Rows and the verdict are plain data (no reactivity): the parent
/// mounts the strip only after submission, and a submitted quiz result
/// never mutates while visible.
#[component]
pub fn QuizFeedbackStrip(
    verdict: LedgerVerdict,
    /// Optional «K/N» counter after the verdict label (multi quizzes).
    #[prop(default = None)]
    count: Option<(usize, usize)>,
    rows: Vec<LedgerRow>,
    /// Known-kanji set for the row text renderer — the same furigana
    /// suppression the options grid uses for the very same strings.
    #[prop(into)]
    known_kanji: Signal<HashSet<char>>,
    #[prop(default = Signal::derive(|| "quiz-feedback-strip".to_string()), into)] test_id: Signal<
        String,
    >,
) -> impl IntoView {
    let i18n = use_i18n();
    let accent = match verdict {
        LedgerVerdict::Correct | LedgerVerdict::LenientPass => "var(--success)",
        LedgerVerdict::Errors | LedgerVerdict::Incorrect => "var(--error)",
        LedgerVerdict::NotAnswered => "var(--fg-muted)",
    };
    let rows_view = (!rows.is_empty()).then(move || {
        view! {
            <div>
                {rows
                    .into_iter()
                    .map(|row| {
                        view! { <LedgerTableRow row known_kanji /> }
                    })
                    .collect_view()
                }
            </div>
        }
        .into_any()
    });
    view! {
        <div
            data-testid=test_id
            class="mt-4 border border-[var(--border-dark)] bg-[var(--bg-aged)]"
            style=format!("border-left: 4px solid {accent}")
        >
            <div class="flex items-center gap-2 px-3 py-2">
                <span
                    class="inline-block w-2.5 h-2.5"
                    style=if verdict == LedgerVerdict::NotAnswered {
                        format!("border: 1px dashed {accent}; background: transparent")
                    } else {
                        format!("background: {accent}")
                    }
                ></span>
                <span class="text-xs font-mono uppercase tracking-[0.1em] font-semibold" style=format!(
                    "color: {accent}"
                )>
                    {verdict_label(i18n, verdict)}
                    <Show when=move || count.is_some()>
                        {move || {
                            let (k, n) = count.unwrap_or((0, 0));
                            format!(" · {k}/{n}")
                        }}
                    </Show>
                </span>
            </div>

            {rows_view}
        </div>
    }
}

#[component]
fn LedgerTableRow(
    row: LedgerRow,
    #[prop(into)] known_kanji: Signal<HashSet<char>>,
) -> impl IntoView {
    let i18n = use_i18n();
    let is_dashed = row.kind == LedgerRowKind::Missed;
    let is_cross = matches!(
        row.kind,
        LedgerRowKind::ExtraSelected | LedgerRowKind::YourAnswerWrong
    );
    let accent = match row.kind {
        LedgerRowKind::SelectedCorrect
        | LedgerRowKind::YourAnswerCorrect
        | LedgerRowKind::CorrectAnswer => Some("var(--success)"),
        LedgerRowKind::ExtraSelected | LedgerRowKind::YourAnswerWrong => Some("var(--error)"),
        LedgerRowKind::Missed => None,
    };
    let (mark, mark_color) = if is_cross {
        ("✕", "var(--error)")
    } else if accent.is_some() {
        ("✓", "var(--success)")
    } else {
        ("", "var(--fg-muted)")
    };
    let strike = if is_cross {
        "line-through decoration-2"
    } else {
        ""
    };
    let text = row.text.clone();
    let tag = row.tag.clone();
    let dashed_border = if is_dashed {
        "border-dashed bg-[var(--bg-secondary)]"
    } else {
        ""
    };
    let row_accent = accent
        .map(|c| format!("border-left: 4px solid {c}"))
        .unwrap_or_default();
    let mark_style = if mark.is_empty() {
        format!("border: 1px dashed {mark_color}; color: {mark_color}")
    } else {
        format!("background: {mark_color}; color: var(--bg-paper)")
    };
    let tag_view = tag.map(|t| {
        view! {
            <span class="text-[10px] font-mono uppercase tracking-[0.1em] px-1.5 py-0.5 border border-[var(--border-dark)] text-[var(--fg-muted)]">
                {t}
            </span>
        }
            .into_any()
    });

    view! {
        <div
            class=format!("flex items-center justify-between gap-2 px-3 py-2 border-t border-[var(--border-dark)] {dashed_border}")
            style=row_accent
        >
            <span class=format!("flex items-center gap-2 min-w-0 {strike}")>
                <span class="text-lg">
                    <MarkdownText
                        content=Signal::derive(move || text.clone())
                        variant=MarkdownVariant::Default
                        known_kanji=known_kanji.get()
                        glue_punctuation=true
                    />
                </span>
                {tag_view}
            </span>
            <span class="flex items-center gap-1.5 shrink-0">
                <span
                    class="inline-flex w-4 h-4 items-center justify-center text-[10px]"
                    style=mark_style
                >
                    {mark}
                </span>
                <span class="text-[10px] font-mono uppercase tracking-[0.1em] text-[var(--fg-muted)]">
                    {row_label(i18n, row.kind)}
                </span>
            </span>
        </div>
    }
}
