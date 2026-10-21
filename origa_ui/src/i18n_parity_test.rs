//! Полный гейт паритета переводов локалей (issue #799).
//!
//! Восстанавливает потерянные проверки locale-части скрипта
//! `scripts/validate_translations.py` (KO/VI-конвейер, коммит 059c514b):
//! leaf-parity, паритет множеств плейсхолдеров и untranslated-гейт для
//! en/ru/ko/vi. leptos_i18n 0.6 не фейлит сборку при расхождении ключей
//! (`emit_diagnostics` даёт только cargo:warning, который `-D warnings` не
//! видит), поэтому инвариант держится этим тестом.
//!
//! Осознанные ужесточения относительно старого locale-гейта: пустые строки и
//! не-строковые листья там не проверялись (`leaf_values` молча пропускал
//! не-строки) — здесь они считаются ошибкой.
//!
//! Дубликаты JSON-ключей не детектируются: serde_json оставляет последний,
//! leptos_i18n ведёт себя так же — паритет со старым скриптом.
//!
//! Allowlist (`is_identity_allowed`) работает по значению en после trim:
//! точный набор из 15 строк старого скрипта плюс разрешающие предикаты
//! (числа/проценты, URL, e-mail, акронимы). Предикаты намеренно разрешающие:
//! короткий CAPS-токен («HOME») пройдёт untranslated-гейт молча — унаследовано
//! от старого скрипта. Новая легитимная identity-строка = правка allowlist
//! разработчиком.

use serde_json::Value;
use std::collections::BTreeMap;

const LOCALES: &[(&str, &str)] = &[
    ("en", include_str!("../locales/en.json")),
    ("ru", include_str!("../locales/ru.json")),
    ("ko", include_str!("../locales/ko.json")),
    ("vi", include_str!("../locales/vi.json")),
];

/// Контракт issue #415 (сохранён дословно из `i18n_counter_keys_test.rs`,
/// модуль поглощён полным гейтом #799): два counter-ключа присутствуют во
/// всех четырёх локалях.
#[test]
fn counter_i18n_keys_present_in_all_four_locales() {
    for (locale, raw) in LOCALES {
        let value: Value = serde_json::from_str(raw).unwrap_or_else(|e| panic!("{locale}: {e}"));
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

/// Leaf-parity: множества листьев ru/ko/vi совпадают с en (в обе стороны).
#[test]
fn i18n_locales_have_leaf_parity_with_en() {
    let maps = locale_maps();
    let (_, en) = &maps[0];
    for (locale, map) in &maps[1..] {
        let missing: Vec<&String> = en
            .keys()
            .filter(|key| !map.contains_key(key.as_str()))
            .collect();
        let extra: Vec<&String> = map
            .keys()
            .filter(|key| !en.contains_key(key.as_str()))
            .collect();
        assert!(
            missing.is_empty(),
            "{locale}: missing keys vs en: {missing:?}"
        );
        assert!(extra.is_empty(), "{locale}: extra keys vs en: {extra:?}");
    }
}

/// Каждый лист — непустая (после trim) строка во всех локалях, включая en.
/// Ужесточение относительно старого гейта, см. док модуля.
#[test]
fn i18n_locale_values_are_non_empty_strings() {
    for (locale, raw) in LOCALES {
        let (_, problems) = locale_map(locale, raw);
        assert!(problems.is_empty(), "{locale}: {problems:?}");
    }
}

/// Паритет множеств плейсхолдеров каждого ключа против en.
#[test]
fn i18n_placeholders_match_en_in_all_locales() {
    let maps = locale_maps();
    let (_, en) = &maps[0];
    for (locale, map) in &maps[1..] {
        for (key, en_value) in en {
            let Some(other) = map.get(key) else {
                continue; // отсутствие ключа ловит leaf-parity
            };
            let mut en_tokens = extract_placeholders(en_value);
            let mut other_tokens = extract_placeholders(other);
            en_tokens.sort();
            other_tokens.sort();
            assert_eq!(
                en_tokens, other_tokens,
                "{locale}/{key}: placeholder mismatch (en {en_value:?} vs {other:?})"
            );
        }
    }
}

/// Untranslated-гейт: значение, совпадающее с en после trim, разрешено только
/// через allowlist. Сравнение по значению en (не перевода) — allowlist один
/// на все локали.
#[test]
fn i18n_no_untranslated_copies_of_en() {
    let maps = locale_maps();
    let (_, en) = &maps[0];
    for (locale, map) in &maps[1..] {
        for (key, en_value) in en {
            let Some(other) = map.get(key) else {
                continue; // отсутствие ключа ловит leaf-parity
            };
            if other.trim() == en_value.trim() && !is_identity_allowed(en_value) {
                panic!(
                    "{locale}/{key}: untranslated copy of en ({other:?}); \
                     если строка легитимно не переводится, добавьте её значение \
                     в IDENTITY_ALLOWLIST_EXACT или предикаты is_identity_allowed"
                );
            }
        }
    }
}

fn locale_map(locale: &str, raw: &str) -> (BTreeMap<String, String>, Vec<String>) {
    let value: Value = serde_json::from_str(raw).unwrap_or_else(|e| panic!("{locale}: {e}"));
    let mut map = BTreeMap::new();
    let mut problems = Vec::new();
    collect_leaves(&value, "", &mut map, &mut problems);
    (map, problems)
}

fn locale_maps() -> Vec<(&'static str, BTreeMap<String, String>)> {
    LOCALES
        .iter()
        .map(|(locale, raw)| {
            let (map, problems) = locale_map(locale, raw);
            assert!(problems.is_empty(), "{locale}: {problems:?}");
            (*locale, map)
        })
        .collect()
}

/// Обходчик листьев: dict → рекурсия; лист обязан быть непустой строкой.
/// Не-строковый лист в карту не попадает (leaf-parity дополнительно сообщит
/// о недостающем ключе).
fn collect_leaves(
    node: &Value,
    prefix: &str,
    out: &mut BTreeMap<String, String>,
    problems: &mut Vec<String>,
) {
    match node {
        Value::Object(map) => {
            for (key, child) in map {
                let child_prefix = if prefix.is_empty() {
                    key.clone()
                } else {
                    format!("{prefix}.{key}")
                };
                collect_leaves(child, &child_prefix, out, problems);
            }
        },
        Value::String(s) => {
            if s.trim().is_empty() {
                problems.push(format!("{prefix}: empty string"));
            }
            out.insert(prefix.to_string(), s.clone());
        },
        other => problems.push(format!(
            "{prefix}: must be a string, got {}",
            json_type_name(other)
        )),
    }
}

fn json_type_name(value: &Value) -> &'static str {
    match value {
        Value::Null => "null",
        Value::Bool(_) => "bool",
        Value::Number(_) => "number",
        Value::String(_) => "string",
        Value::Array(_) => "array",
        Value::Object(_) => "object",
    }
}

/// Множество плейсхолдеров значения. Семантика в точности как у Python-regex
/// `\{[^}]+\}` из старого гейта: левый-жадный непересекающийся матч, минимум
/// один символ между скобками. Поэтому `{{count}}` даёт артефактный токен
/// `{{count}`, а `{count}` — `{count}`: дрейф между этими формами ловится.
/// `{}` (пустые скобки) не матчится.
fn extract_placeholders(value: &str) -> Vec<String> {
    let bytes = value.as_bytes();
    let mut tokens = Vec::new();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'{' {
            if let Some(rel) = bytes[i + 1..].iter().position(|&b| b == b'}') {
                if rel >= 1 {
                    let end = i + 1 + rel;
                    // Границы i и end — ASCII-символы, разрезание UTF-8 безопасно.
                    tokens.push(value[i..=end].to_string());
                    i = end + 1;
                    continue;
                }
            }
        }
        i += 1;
    }
    tokens
}

/// Строки, легитимно совпадающие с en во всех локалях: бренды, технические
/// токены, уровни JLPT. Полный набор (15) из старого скрипта.
const IDENTITY_ALLOWLIST_EXACT: &[&str] = &[
    "samurai",
    "haiku",
    "Anki",
    "English",
    "N5",
    "N4",
    "N3",
    "N2",
    "N1",
    "[Enter]",
    "Email",
    "Commit:",
    "{} / {}",
    "Deim • ~50 MB",
    "Parseq • ~100 MB",
];

fn is_identity_allowed(value: &str) -> bool {
    let value = value.trim();
    IDENTITY_ALLOWLIST_EXACT.contains(&value)
        || is_numberish(value)
        || is_url(value)
        || is_email(value)
        || is_acronym(value)
}

/// `^\d[\d.,:%/]*$` — числа, проценты, диапазоны.
fn is_numberish(value: &str) -> bool {
    let mut chars = value.chars();
    matches!(chars.next(), Some(first) if first.is_ascii_digit())
        && chars.all(|c| c.is_ascii_digit() || matches!(c, '.' | ',' | ':' | '%' | '/'))
}

/// `^https?://\S+$`
fn is_url(value: &str) -> bool {
    (value.starts_with("http://") || value.starts_with("https://"))
        && !value.chars().any(char::is_whitespace)
}

/// `^[\w.+-]+@[\w-]+\.[\w.]+$` (строже: `\w` только ASCII — разрешающая
/// сторона уже, false positive добавляется в allowlist вручную).
fn is_email(value: &str) -> bool {
    if value.matches('@').count() != 1 {
        return false;
    }
    let Some((local, domain)) = value.split_once('@') else {
        return false;
    };
    if local.is_empty() || domain.is_empty() {
        return false;
    }
    let local_ok = local
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '+' | '-' | '_'));
    let domain_ok = domain.contains('.')
        && domain
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '-' | '_'));
    local_ok && domain_ok
}

/// `^[A-Z0-9]{1,4}[-\d]*$` — акронимы и коды моделей (APK, X2, N3).
/// Намеренно разрешающая: короткий CAPS-токен пройдёт молча, см. док модуля.
fn is_acronym(value: &str) -> bool {
    let mut chars = value.chars();
    let head_len = chars
        .by_ref()
        .take_while(|c| c.is_ascii_uppercase() || c.is_ascii_digit())
        .count();
    (1..=4).contains(&head_len) && chars.all(|c| c == '-' || c.is_ascii_digit())
}

#[test]
fn placeholder_scanner_reproduces_reference_regex_semantics() {
    // Векторы зафиксированы против re.findall(r"\{[^}]+\}", ...):
    // `{{count}}` → `{{count}` (артефакт левого-жадного матча), `{}` → ничего.
    let cases: &[(&str, &[&str])] = &[
        ("{{count}}", &["{{count}"]),
        ("{count}", &["{count}"]),
        ("{}", &[]),
        ("{} / {}", &[]),
        ("{:.0}", &["{:.0}"]),
        ("{a} и {b}", &["{a}", "{b}"]),
        ("{{accuracy}} {{count}}", &["{{accuracy}", "{{count}"]),
        ("brace without close {", &[]),
        ("close without open }", &[]),
    ];
    for (input, expected) in cases {
        let mut actual = extract_placeholders(input);
        let mut expected: Vec<String> = expected.iter().map(|s| (*s).to_string()).collect();
        actual.sort();
        expected.sort();
        assert_eq!(actual, expected, "input: {input:?}");
    }
}

#[test]
fn leaf_walker_flags_non_string_and_empty_leaves() {
    let node = serde_json::json!({
        "section": { "text": "value" },
        "empty": "   ",
        "blank": "",
        "number": 5,
        "array": ["a"]
    });
    let mut map = BTreeMap::new();
    let mut problems = Vec::new();
    collect_leaves(&node, "", &mut map, &mut problems);
    // Пустые строки остаются в карте (паритет ключей сохраняется) — их ловит
    // problems; из карты исключены только не-строковые листья.
    assert_eq!(
        map.keys().collect::<Vec<_>>(),
        ["blank", "empty", "section.text"]
    );
    assert_eq!(
        problems,
        [
            "array: must be a string, got array",
            "blank: empty string",
            "empty: empty string",
            "number: must be a string, got number",
        ]
    );
}

#[test]
fn identity_allowlist_predicates_match_reference_behaviour() {
    // Точный набор.
    for value in IDENTITY_ALLOWLIST_EXACT {
        assert!(is_identity_allowed(value), "exact allowlist: {value:?}");
    }
    // Предикаты: разрешающие стороны.
    for value in [
        "N3",
        "50%",
        "1,000",
        "10:30",
        "https://origa.uwuwu.net/docs",
        "http://x",
        "user@example.com",
        "user+tag@mail.example.org",
        "APK",
        "X2",
        "A-1",
    ] {
        assert!(is_identity_allowed(value), "predicate: {value:?}");
    }
    // Запретные стороны: настоящий untranslated-текст не проходит.
    for value in [
        "Back",
        "Loading…",
        "50 процентов",
        "mailto:user@example.com",
        "user@localhost",
        "ABC12345",
        "учёба",
    ] {
        assert!(!is_identity_allowed(value), "must reject: {value:?}");
    }
}
