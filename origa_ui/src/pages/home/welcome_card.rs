use crate::i18n::{Locale, td_string, use_i18n};
use leptos::prelude::*;
use leptos_icons::Icon;
use leptos_router::components::A;

/// Time-of-day band that selects the greeting key. Boundaries: 5–11
/// morning, 12–17 afternoon, 18–22 evening, otherwise night.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum DayPeriod {
    Morning,
    Afternoon,
    Evening,
    Night,
}

fn day_period(hours: u8) -> DayPeriod {
    match hours {
        5..=11 => DayPeriod::Morning,
        12..=17 => DayPeriod::Afternoon,
        18..=22 => DayPeriod::Evening,
        _ => DayPeriod::Night,
    }
}

/// Locale-resolved greeting: every locale ships its own key, so Korean
/// and Vietnamese no longer fall through to the English string.
fn greeting_text(locale: Locale, period: DayPeriod) -> String {
    match period {
        DayPeriod::Morning => td_string!(locale, home.greeting_morning),
        DayPeriod::Afternoon => td_string!(locale, home.greeting_afternoon),
        DayPeriod::Evening => td_string!(locale, home.greeting_evening),
        DayPeriod::Night => td_string!(locale, home.greeting_night),
    }
    .to_string()
}

#[component]
pub fn WelcomeCard(
    username: Signal<String>,
    #[prop(optional, into)] test_id: Signal<String>,
) -> impl IntoView {
    let i18n = use_i18n();

    let test_id_val = move || {
        let val = test_id.get();
        if val.is_empty() { None } else { Some(val) }
    };

    let lesson_test_id = Signal::derive(move || {
        let val = test_id.get();
        if val.is_empty() {
            "lesson-buttons-lesson".to_string()
        } else {
            format!("{}-lesson", val)
        }
    });

    view! {
        <div class="py-4 sm:py-6" data-testid=test_id_val>
            <div class="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-4">
                <div class="font-serif text-2xl sm:text-3xl text-[var(--fg-black)]">
                    {move || {
                        let hours = js_sys::Date::new_0().get_hours() as u8;
                        greeting_text(i18n.get_locale(), day_period(hours))
                    }} ", "
                    <span class="text-[var(--accent-olive)]">{move || username.get()}</span>
                    ". "
                    <span class="text-[var(--fg-muted)]">
                        {move || td_string!(i18n.get_locale(), home.welcome_subline)}
                    </span>
                </div>
                <div class="shrink-0">
                    <A href="/lesson">
                        <button
                            class="btn btn-olive flex items-center justify-center gap-2 w-full sm:w-auto sm:min-w-[280px] px-6 py-3 sm:px-10 sm:py-2.5 uppercase"
                            data-testid=move || Some(lesson_test_id.get())
                        >
                            <Icon icon=icondata::LuBookOpen width="16" height="16" />
                            {move || td_string!(i18n.get_locale(), home.lesson)}
                        </button>
                    </A>
                </div>
            </div>
        </div>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[rstest::rstest]
    #[case::night_before_dawn(4, DayPeriod::Night)]
    #[case::morning_start(5, DayPeriod::Morning)]
    #[case::morning_end(11, DayPeriod::Morning)]
    #[case::afternoon_start(12, DayPeriod::Afternoon)]
    #[case::afternoon_end(17, DayPeriod::Afternoon)]
    #[case::evening_start(18, DayPeriod::Evening)]
    #[case::evening_end(22, DayPeriod::Evening)]
    #[case::late_night(23, DayPeriod::Night)]
    #[case::midnight(0, DayPeriod::Night)]
    fn day_period_bands_follow_wall_clock_hours(#[case] hours: u8, #[case] expected: DayPeriod) {
        assert_eq!(day_period(hours), expected);
    }

    /// The regression this fix guards: Korean and Vietnamese used to fall
    /// into the English branch of the hardcoded greeting and button.
    #[rstest::rstest]
    #[case::korean(Locale::ko, "좋은 아침이에요")]
    #[case::vietnamese(Locale::vi, "Chào buổi sáng")]
    #[case::russian(Locale::ru, "Доброе утро")]
    #[case::english(Locale::en, "Good morning")]
    fn morning_greeting_is_translated_for_every_locale(
        #[case] locale: Locale,
        #[case] expected: &str,
    ) {
        assert_eq!(greeting_text(locale, DayPeriod::Morning), expected);
    }

    #[rstest::rstest]
    #[case::korean(Locale::ko, "좋은 밤 되세요")]
    #[case::vietnamese(Locale::vi, "Chúc ngủ ngon")]
    fn night_greeting_is_not_english_for_korean_and_vietnamese(
        #[case] locale: Locale,
        #[case] expected: &str,
    ) {
        assert_eq!(greeting_text(locale, DayPeriod::Night), expected);
    }
}
