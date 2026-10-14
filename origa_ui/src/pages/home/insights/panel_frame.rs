/// Класс фразы вердикта по статусу: HOLD — приглушённый оливковый,
/// DIP — терракотовый, нет данных — серый.
pub(crate) fn verdict_class_by_kind(
    kind: Option<origa::domain::insights::WeekVerdictKind>,
) -> &'static str {
    use origa::domain::insights::WeekVerdictKind;
    match kind {
        None => "heading-h3 text-[var(--fg-muted)]",
        Some(WeekVerdictKind::Dipping) => "heading-h3 text-[var(--error)]",
        Some(WeekVerdictKind::Holding) => "heading-h3 text-[var(--success)]",
    }
}
