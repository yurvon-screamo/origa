//! Паритет counter-ключей во всех четырёх локалях (issue #415):
//! lesson.counter, onboarding.scoring.section.counter.

#[test]
fn counter_i18n_keys_present_in_all_four_locales() {
    let locales: &[(&str, &str)] = &[
        ("en", include_str!("../locales/en.json")),
        ("ru", include_str!("../locales/ru.json")),
        ("ko", include_str!("../locales/ko.json")),
        ("vi", include_str!("../locales/vi.json")),
    ];
    for (locale, raw) in locales {
        let value: serde_json::Value =
            serde_json::from_str(raw).unwrap_or_else(|e| panic!("{locale}: {e}"));
        let paths: [&[&str]; 2] = [
            &["lesson", "counter"],
            &["onboarding", "scoring", "section", "counter"],
        ];
        for path in paths {
            let missing = path
                .iter()
                .try_fold(&value, |node, key| node.get(key))
                .is_none();
            assert!(
                !missing,
                "{locale}: missing non-empty key {}",
                path.join(".")
            );
        }
    }
}
