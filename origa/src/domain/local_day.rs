//! Локальная граница суток для дневных сущностей домена.
//!
//! Дневные лимиты, дневная история и отметка «знаю» живут в календарных
//! днях пользователя: полночь границы — локальная, не UTC. Компромисс с
//! Clean Architecture (первое вхождение `chrono::Local` в crate): Origa —
//! single-user клиент, локальная полночь — продуктовое требование.
//! Ambient-зона изолирована в [`local_offset`]: граничные тесты не зависят
//! от ambient (явные `FixedOffset`/`today_start`), инвариант-тесты
//! согласованы с прод-кодом через общий ambient-источник.

use chrono::{DateTime, FixedOffset, Utc};

/// UTC-инстант полуночи локальных суток, содержащих `now`, в зоне `offset`.
/// Чистая функция: детерминирована, тестируема с фиксированным офсетом.
pub fn start_of_day(offset: FixedOffset, now: DateTime<Utc>) -> DateTime<Utc> {
    let local_date = now.with_timezone(&offset).date_naive();
    let midnight = local_date
        .and_hms_opt(0, 0, 0)
        .expect("midnight is a valid naive time");
    midnight
        .and_local_timezone(offset)
        .single()
        .expect("fixed offsets have no DST ambiguity")
        .with_timezone(&Utc)
}

/// Оффсет локальной зоны процесса. WASM: зона браузера (`chrono/wasmbind`
/// через `js_sys`); native: зона ОС. Известное ограничение: офсет
/// снапшотится на момент вызова — в день DST-перехода граница суток
/// может сместиться до часа (для зон без DST, как RU/JP, нерелевантно).
pub fn local_offset() -> FixedOffset {
    *chrono::Local::now().offset()
}

/// Старт текущих локальных суток как UTC-инстант. Единственный источник
/// «сейчас» — `Local::now()`: и момент, и офсет берутся из него.
pub fn today_start() -> DateTime<Utc> {
    let local_now = chrono::Local::now();
    start_of_day(*local_now.offset(), local_now.with_timezone(&Utc))
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Duration, TimeZone, Timelike};
    use rstest::rstest;

    fn instant(date: &str, h: u32, min: u32) -> DateTime<Utc> {
        let (y, m, d) = (
            date[0..4].parse::<i32>().unwrap(),
            date[5..7].parse::<u32>().unwrap(),
            date[8..10].parse::<u32>().unwrap(),
        );
        Utc.with_ymd_and_hms(y, m, d, h, min, 0).unwrap()
    }

    fn offset(hours: i32) -> FixedOffset {
        FixedOffset::east_opt(hours * 3600).unwrap()
    }

    #[rstest]
    // Москва +3: 01:30 локально 20-го (22:30Z 19-го) → полночь 19-го 21:00Z
    #[case::moscow_after_local_midnight(
        offset(3),
        instant("2026-09-19", 22, 30),
        instant("2026-09-19", 21, 0)
    )]
    // Кирибати +14: 05:00 локально 20-го (15:00Z 19-го) → полночь 19-го 10:00Z
    #[case::easternmost_zone(
        offset(14),
        instant("2026-09-19", 15, 0),
        instant("2026-09-19", 10, 0)
    )]
    // Самоа −11: 08:00 локально 19-го (19:00Z 19-го) → полночь 19-го 11:00Z
    #[case::westernmost_zone(
        offset(-11),
        instant("2026-09-19", 19, 0),
        instant("2026-09-19", 11, 0),
    )]
    // UTC: полночь суток — сам инстант суток
    #[case::utc_zone(offset(0), instant("2026-09-20", 6, 15), instant("2026-09-20", 0, 0))]
    fn start_of_day_returns_local_midnight_as_utc_instant(
        #[case] offset: FixedOffset,
        #[case] now: DateTime<Utc>,
        #[case] expected: DateTime<Utc>,
    ) {
        assert_eq!(start_of_day(offset, now), expected);
    }

    #[rstest]
    #[case::moscow(offset(3))]
    #[case::kiribati(offset(14))]
    #[case::samoa(offset(-11))]
    #[case::utc(offset(0))]
    fn start_of_day_bounds_instant_within_its_local_day(#[case] offset: FixedOffset) {
        let now = Utc::now();

        let start = start_of_day(offset, now);

        assert!(start <= now, "day start must not be after `now`");
        assert!(
            now - start < Duration::hours(24),
            "`now` must fall within the day started at `start`"
        );
        assert_eq!(
            start.with_timezone(&offset).date_naive(),
            now.with_timezone(&offset).date_naive(),
            "day start and `now` must share the local calendar date"
        );
        assert_eq!(
            start.with_timezone(&offset).num_seconds_from_midnight(),
            0,
            "day start must be local midnight"
        );
    }
}
