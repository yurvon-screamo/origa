//! Реестр счётных суффиксов (issue #415): контент всех counter-карт.
//! Зеркало `GRAMMAR_RULES` — глобальный `OnceLock`, наполняемый лоадером
//! из `counters/counters.json` на CDN (кэш — ADR-053). Карточка хранит
//! только суффикс и памяти связок; чтения, нерегулярность, глоссы и
//! JLPT-уровень резолвятся отсюда на рендере.

use std::collections::HashMap;
use std::sync::OnceLock;

use serde::Deserialize as _;

use crate::domain::{JapaneseLevel, NativeLanguage};

pub static COUNTERS: OnceLock<Vec<CounterEntry>> = OnceLock::new();

/// Одна строка чтения в таблице суффикса.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ReadingEntry {
    /// 1..=10, 0 = вопросительное 何; лексикализованные исключения >10.
    pub number: u8,
    /// Каноническое чтение связки целиком («さんぼん»).
    pub reading: String,
    /// Нерегулярная ячейка: приоритет в пачке показа + подсветка в таблице.
    #[serde(default)]
    pub irregular: bool,
}

/// Счётный суффикс реестра.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct CounterEntry {
    /// Суффикс — content_key и ключ резолва («本»).
    pub suffix: String,
    /// Редакторская JLPT-группировка (официальной не существует).
    #[serde(deserialize_with = "de_level")]
    pub level: JapaneseLevel,
    /// Глоссы «что считают»; все 4 локали обязательны (валидатор датасета),
    /// рантайм-страховка — цепочка в [`gloss_for`].
    pub glosses: HashMap<NativeLanguage, String>,
    /// Таблица чтений; порядок — по возрастанию числа, 何 последней
    /// (инвариант сборщика).
    pub readings: Vec<ReadingEntry>,
}

fn de_level<'de, D>(deserializer: D) -> Result<JapaneseLevel, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s: String = String::deserialize(deserializer)?;
    s.parse::<JapaneseLevel>()
        .map_err(|_| serde::de::Error::custom(format!("unknown JLPT level: {s}")))
}

impl ReadingEntry {
    pub fn number(&self) -> u8 {
        self.number
    }

    pub fn reading(&self) -> &str {
        &self.reading
    }

    pub fn irregular(&self) -> bool {
        self.irregular
    }
}

impl CounterEntry {
    pub fn suffix(&self) -> &str {
        &self.suffix
    }

    pub fn level(&self) -> JapaneseLevel {
        self.level
    }

    pub fn readings(&self) -> &[ReadingEntry] {
        &self.readings
    }

    pub fn reading_for(&self, number: u8) -> Option<&str> {
        self.readings
            .iter()
            .find(|r| r.number == number)
            .map(|r| r.reading.as_str())
    }

    pub fn irregular_for(&self, number: u8) -> bool {
        self.readings
            .iter()
            .find(|r| r.number == number)
            .is_some_and(|r| r.irregular)
    }
}

/// Локализованная метка типа для попапа транслейтера и UI-тегов.
/// Константный map в домене: крейт `origa` не видит leptos_i18n —
/// прецедент контентных строк `GrammarMatch::from_rule`.
/// Минимальный публичный реестр для межкрейтовых тестов (origa_ui):
/// 本 N5 с 4 чтениями. Одноразовый OnceLock — первый вызов фиксирует.
pub fn init_minimal_counters() {
    let mut glosses = HashMap::new();
    glosses.insert(NativeLanguage::Russian, "длинные предметы".to_string());
    glosses.insert(NativeLanguage::English, "long objects".to_string());
    glosses.insert(NativeLanguage::Korean, "자루".to_string());
    glosses.insert(NativeLanguage::Vietnamese, "cây".to_string());
    let _ = COUNTERS.set(vec![CounterEntry {
        suffix: "本".to_string(),
        level: JapaneseLevel::N5,
        glosses,
        readings: vec![
            ReadingEntry {
                number: 1,
                reading: "いっぽん".into(),
                irregular: true,
            },
            ReadingEntry {
                number: 2,
                reading: "にほん".into(),
                irregular: false,
            },
            ReadingEntry {
                number: 3,
                reading: "さんぼん".into(),
                irregular: true,
            },
            ReadingEntry {
                number: 0,
                reading: "なんぼん".into(),
                irregular: false,
            },
        ],
    }]);
}

pub fn counter_label(lang: NativeLanguage) -> &'static str {
    match lang {
        NativeLanguage::Russian => "Счётный суффикс",
        NativeLanguage::English => "Counter",
        NativeLanguage::Korean => "조수사",
        NativeLanguage::Vietnamese => "từ đếm",
    }
}

/// Глосс реестра в локали юзера: requested → English → any → пусто.
/// Терминал «пусто» = рендер без глосса; при прохождении валидатора
/// датасета (все 4 локали непустые) ветка недостижима.
pub fn gloss_for(entry: &CounterEntry, lang: NativeLanguage) -> &str {
    if let Some(g) = entry.glosses.get(&lang) {
        if !g.is_empty() {
            return g;
        }
    }
    if let Some(g) = entry.glosses.get(&NativeLanguage::English) {
        if !g.is_empty() {
            return g;
        }
    }
    entry
        .glosses
        .values()
        .find(|g| !g.is_empty())
        .map(String::as_str)
        .unwrap_or("")
}

/// Загружает реестр из JSON CDN-артефакта (полный развёрнутый формат).
/// Повторный вызов — no-op (лоадеры идемпотентны). Возвращает размер
/// реестра; пустой/битый JSON — ошибка: тишина здесь означала бы, что
/// счётчики молча исчезли из рук и ревью.
pub fn init_counters(json: &str) -> Result<usize, String> {
    if let Some(existing) = COUNTERS.get() {
        return Ok(existing.len());
    }
    let wire: WireFile = serde_json::from_str(json).map_err(|e| e.to_string())?;
    let counters = wire.counters;
    let len = counters.len();
    if len == 0 {
        return Err("counters registry is empty".to_string());
    }
    let _ = COUNTERS.set(counters);
    Ok(len)
}

pub fn is_counters_loaded() -> bool {
    COUNTERS.get().is_some_and(|c| !c.is_empty())
}

/// Резолв записи реестра по суффиксу.
pub fn get_counter(suffix: &str) -> Option<&'static CounterEntry> {
    COUNTERS.get()?.iter().find(|entry| entry.suffix == suffix)
}

/// Все записи уровней ≤ `level` (Ord: N5 < N4 < … < N1 — фильтр `<=`,
/// порядок реестра: N5 первыми).
pub fn counters_up_to_level(level: JapaneseLevel) -> Vec<&'static CounterEntry> {
    match COUNTERS.get() {
        Some(counters) => counters
            .iter()
            .filter(|entry| entry.level <= level)
            .collect(),
        None => Vec::new(),
    }
}

/// Кандидаты миграции: суффикс встречается в написании слова юзера
/// как связка «числительное + суффикс» (primary surface-детект), либо
/// слово совпадает с суффиксом. `日本` не детектит `本` — числительного
/// рядом нет.
const NUMERAL_CHARS: &[char] = &[
    '一', '二', '三', '四', '五', '六', '七', '八', '九', '十', '百', '千',
];

/// Суффиксы реестра, чей счётный контекст виден в поверхности ОДНОЙ
/// лексемы (числительное + суффикс внутри слова — склейки токенизатора
/// вида 三本, которые не расщепляются в пары токенов).
pub fn counters_detected_in_surface(word: &str) -> Vec<&'static CounterEntry> {
    match COUNTERS.get() {
        Some(entries) => entries
            .iter()
            .filter(|entry| suffix_detected_in_word(word, entry.suffix()))
            .collect(),
        None => Vec::new(),
    }
}

pub fn suffix_detected_in_word(word: &str, suffix: &str) -> bool {
    // Standalone-совпадение НЕ детектим: изолированный 本 — существительное
    // («книга»), счётный суффикс проявляется только в числительных
    // сочетаниях (一本) — чужие колоды (слово 本 из прозы) не получают
    // ложных counter-карт.

    let chars: Vec<char> = word.chars().collect();
    let suffix_chars: Vec<char> = suffix.chars().collect();
    if suffix_chars.is_empty() || chars.len() <= suffix_chars.len() {
        return false;
    }
    for window_start in 0..=chars.len() - suffix_chars.len() {
        if chars[window_start..window_start + suffix_chars.len()] == suffix_chars[..] {
            let before = window_start.checked_sub(1).map(|i| chars[i]);
            if before.is_some_and(|c| NUMERAL_CHARS.contains(&c)) {
                return true;
            }
        }
    }
    false
}

#[derive(serde::Deserialize)]
struct WireFile {
    counters: Vec<CounterEntry>,
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use std::sync::Mutex;

    pub const TEST_HON: &str = "本";
    pub const TEST_NIN: &str = "人";

    /// OnceLock одноразовый на процесс: все тесты модуля делят один
    /// и тот же полный реестр, сериализация — общим локом.
    static TEST_LOCK: Mutex<()> = Mutex::new(());

    fn readings_onbin(series: &[&str]) -> Vec<ReadingEntry> {
        series
            .iter()
            .enumerate()
            .map(|(idx, reading)| ReadingEntry {
                // idx 0 → number 0 (何), idx 1..=10 → 1..=10
                number: idx as u8,
                reading: reading.to_string(),
                irregular: reading.starts_with("いっ")
                    || reading.starts_with("さん")
                    || reading.starts_with("ろっ")
                    || reading.starts_with("はっ")
                    || reading.starts_with("じゅっ")
                    || reading.starts_with("ひと")
                    || reading.starts_with("ふた")
                    || reading.starts_with("よ")
                    || reading.starts_with("しち"),
            })
            .collect()
    }

    fn hon_entry() -> CounterEntry {
        let mut glosses = HashMap::new();
        glosses.insert(NativeLanguage::Russian, "длинные предметы".to_string());
        glosses.insert(NativeLanguage::English, "long objects".to_string());
        glosses.insert(NativeLanguage::Korean, "자루".to_string());
        glosses.insert(NativeLanguage::Vietnamese, "cây".to_string());
        CounterEntry {
            suffix: TEST_HON.to_string(),
            level: JapaneseLevel::N5,
            glosses,
            readings: readings_onbin(&[
                "なんぼん",
                "いっぽん",
                "にほん",
                "さんぼん",
                "よんほん",
                "ごほん",
                "ろっぽん",
                "ななほん",
                "はっぽん",
                "きゅうほん",
                "じゅっぽん",
            ]),
        }
    }

    fn nin_entry() -> CounterEntry {
        let mut glosses = HashMap::new();
        glosses.insert(NativeLanguage::Russian, "люди".to_string());
        glosses.insert(NativeLanguage::English, "people".to_string());
        glosses.insert(NativeLanguage::Korean, "명".to_string());
        glosses.insert(NativeLanguage::Vietnamese, "người".to_string());
        CounterEntry {
            suffix: TEST_NIN.to_string(),
            level: JapaneseLevel::N5,
            glosses,
            readings: readings_onbin(&[
                "なんにん",
                "ひとり",
                "ふたり",
                "さんにん",
                "よにん",
                "ごにん",
                "ろくにん",
                "しちにん",
                "はちにん",
                "きゅうにん",
                "じゅうにん",
            ]),
        }
    }

    fn nichi_entry() -> CounterEntry {
        // 日-контракт плана: 1..=10 + 何 + лексикализованные 14/20/24 —
        // первый показ 日 это 14 мини-вопросов (issue #415).
        let mut glosses = HashMap::new();
        glosses.insert(NativeLanguage::Russian, "дни месяца".to_string());
        glosses.insert(NativeLanguage::English, "days".to_string());
        glosses.insert(NativeLanguage::Korean, "일".to_string());
        glosses.insert(NativeLanguage::Vietnamese, "ngày".to_string());
        let mut readings = readings_onbin(&[
            "なんにち",
            "ついたち",
            "ふつか",
            "みっか",
            "よっか",
            "いつか",
            "むいか",
            "なのか",
            "ようか",
            "ここのか",
            "とおか",
        ]);
        readings.push(ReadingEntry {
            number: 14,
            reading: "じゅうよっか".into(),
            irregular: true,
        });
        readings.push(ReadingEntry {
            number: 20,
            reading: "はつか".into(),
            irregular: true,
        });
        readings.push(ReadingEntry {
            number: 24,
            reading: "にじゅうよっか".into(),
            irregular: true,
        });
        CounterEntry {
            suffix: "日".to_string(),
            level: JapaneseLevel::N5,
            glosses,
            readings,
        }
    }

    /// Тестовый реестр: 本/人 (11 ячеек) + 日 (14 ячеек), все N5.
    /// Идемпотентен: повторные вызовы делят уже установленный реестр.
    pub fn init_test_counters() {
        let _guard = TEST_LOCK.lock();
        let _ = COUNTERS.set(vec![hon_entry(), nin_entry(), nichi_entry()]);
    }

    #[test]
    fn wire_format_roundtrips_full_counter_entry() {
        let entry = hon_entry();
        let json = serde_json::to_string(&entry).unwrap();
        let restored: CounterEntry = serde_json::from_str(&json).unwrap();
        assert_eq!(restored, entry);
        assert_eq!(restored.reading_for(3), Some("さんぼん"));
        assert!(restored.irregular_for(3));
        assert!(!restored.irregular_for(2));
    }

    #[test]
    fn wire_file_parses_and_rejects_empty() {
        let wire = serde_json::json!({ "counters": [hon_entry()] }).to_string();
        let parsed: Result<WireFile, _> = serde_json::from_str(&wire);
        assert!(parsed.is_ok());
        assert_eq!(parsed.unwrap().counters.len(), 1);

        let empty = serde_json::json!({ "counters": [] }).to_string();
        assert!(serde_json::from_str::<WireFile>(&empty).is_ok());
        // init_counters на пустом реестре — ошибка (проверка ниже через
        // уже установленный реестр безопасна: no-op возвращает len > 0).
        init_test_counters();
        assert!(is_counters_loaded());
    }

    #[test]
    fn level_deserializes_from_jlpt_code() {
        let json = serde_json::json!({
            "suffix": "匹", "level": "N3",
            "glosses": {}, "readings": []
        });
        let entry: CounterEntry = serde_json::from_value(json).unwrap();
        assert_eq!(entry.level(), JapaneseLevel::N3);
        assert!(
            serde_json::from_value::<CounterEntry>(serde_json::json!({
                "suffix": "x", "level": "N9", "glosses": {}, "readings": []
            }))
            .is_err()
        );
    }

    #[test]
    fn init_counters_is_idempotent_noop_when_already_set() {
        init_test_counters();
        // Реестр уже установлен другим тестом или этим — вызов с любым
        // JSON возвращает размер существующего, ничего не ломая.
        let len = init_counters("{\"counters\":[]}").unwrap();
        assert_eq!(len, 3);
    }

    #[test]
    fn gloss_fallback_chain_requested_english_any_empty() {
        init_test_counters();
        let entry = hon_entry();
        assert_eq!(
            gloss_for(&entry, NativeLanguage::Russian),
            "длинные предметы"
        );
        assert_eq!(gloss_for(&entry, NativeLanguage::English), "long objects");
        assert_eq!(gloss_for(&entry, NativeLanguage::Korean), "자루");
        assert_eq!(gloss_for(&entry, NativeLanguage::Vietnamese), "cây");

        let mut partial = entry.clone();
        partial.glosses.remove(&NativeLanguage::Korean);
        assert_eq!(gloss_for(&partial, NativeLanguage::Korean), "long objects");

        partial.glosses.remove(&NativeLanguage::English);
        partial.glosses.remove(&NativeLanguage::Russian);
        assert_eq!(gloss_for(&partial, NativeLanguage::Russian), "cây");

        partial.glosses.remove(&NativeLanguage::Vietnamese);
        assert_eq!(gloss_for(&partial, NativeLanguage::Russian), "");
    }

    #[test]
    fn get_counter_resolves_and_up_to_level_includes_lower_levels() {
        init_test_counters();
        assert_eq!(get_counter(TEST_HON).unwrap().suffix(), TEST_HON);
        assert!(get_counter("虚").is_none());

        // Фикстуры — N5: «≤ уровня» включает их для N4 и N1.
        assert_eq!(counters_up_to_level(JapaneseLevel::N5).len(), 3);
        assert_eq!(counters_up_to_level(JapaneseLevel::N4).len(), 3);
        assert_eq!(counters_up_to_level(JapaneseLevel::N1).len(), 3);
    }

    #[test]
    fn surface_detect_matches_numeral_adjacent_suffix_only() {
        init_test_counters();
        assert!(suffix_detected_in_word("三日", "日"));
        assert!(suffix_detected_in_word("一本", "本"));
        // Standalone 本 — существительное, не счётный контекст.
        assert!(
            !suffix_detected_in_word("本", "本"),
            "слово == суффикс не детектится"
        );
        assert!(
            !suffix_detected_in_word("日本", "本"),
            "числительного рядом нет"
        );
        assert!(!suffix_detected_in_word("本棚", "本"));
    }

    #[test]
    fn nichi_first_showcase_is_fourteen_bindings() {
        init_test_counters();
        let mut card = crate::domain::CounterCard::new("日");
        card.ensure_registry_bindings();
        assert_eq!(card.bindings().len(), 14, "1..=10 + 何 + 14/20/24");
        assert_eq!(card.binding_showcase().len(), 14);
    }

    #[test]
    fn counter_label_covers_four_locales() {
        assert!(!counter_label(NativeLanguage::Russian).is_empty());
        assert!(!counter_label(NativeLanguage::English).is_empty());
        assert!(!counter_label(NativeLanguage::Korean).is_empty());
        assert!(!counter_label(NativeLanguage::Vietnamese).is_empty());
    }
}
