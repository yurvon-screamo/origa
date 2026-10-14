/// Класс фразы вердикта по статусу: HOLD — приглушённый оливковый,
/// DIP — терракотовый, нет данных — серый.
pub(crate) fn verdict_class_by_kind(
    kind: Option<origa::domain::insights::WeekVerdictKind>,
) -> &'static str {
    use origa::domain::insights::WeekVerdictKind;
    match kind {
        None => "heading-h2 text-[var(--fg-muted)]",
        Some(WeekVerdictKind::Dipping) => "heading-h2 text-[var(--error)]",
        Some(WeekVerdictKind::Holding) => "heading-h2 text-[var(--success)]",
    }
}
