use crate::repository::HybridUserRepository;
use leptos::{prelude::*, task::spawn_local};
use origa::domain::{AcquaintanceHand, AcquaintanceSubphase, NativeLanguage};
use origa::use_cases::CompleteAcquaintanceHandUseCase;
use std::collections::HashSet;
use ulid::Ulid;

use super::grammar_example::grammar_training_front_text;
use super::kanji_card_details::RadicalDisplay;
use crate::ui_components::ReadingItem;

/// Аудио-фронт тренировки: живёт ТОЛЬКО в подфазе «яп→рус» (Forward) —
/// монета 50/50 бросается заново на КАЖДЫЙ показ: половина карт звучит
/// без текста (юзер вспоминает перевод на слух), половина — текстовый
/// фронт. Подфаза «рус→яп» (Reverse) — всегда текстовый реверсед, монета
/// там не бросается (решение владельца, 2026-09-15).
///
/// Чистое решение одного броска: `true` (аудио-фронт) только для слова в
/// Forward-подфазе при выпавшем roll и реальной озвучиваемости слова.
/// Свободная функция: случайность и доступность аудио живут снаружи
/// (UI), а инвариант «аудио-фронт недостижим на Reverse и несловесных
/// картах» закреплён юнит-матрицей, а не рассыпан по Effect'у.
pub fn resolve_audio_front(
    is_word: bool,
    subphase: Option<AcquaintanceSubphase>,
    roll: bool,
    audio_available: bool,
) -> bool {
    is_word && subphase == Some(AcquaintanceSubphase::Forward) && roll && audio_available
}

/// Вид озвучиваемой карты: управляет видимостью кнопки озвучки в
/// тренировке. У слова японская сторона может скрываться фронтом
/// (Reverse, аудио-фронт Forward — кнопка подсказывала бы ответ голосом);
/// у грамматики JP-строка фронта (пример/паттерн) видна в любой подфазе
/// и на ответе. Кандзи и счётные суффиксы беззвучны (#808) — до вида не
/// доходят: [`AcquaintanceSlideData::audio`] даёт для них `None`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VoicedKind {
    /// Слово: JP-сторону скрывают Reverse-фронт и аудио-фронт Forward.
    Word,
    /// Грамматика: JP-строка фронта на экране всегда.
    FrontAlwaysVisible,
}

/// Стадии руки знакомства на странице урока (docs/acquaintance-mode.md):
/// показ → тренировка → переходный экран → обычное ревью. `Inactive` —
/// руки нет (пул пуст / лимит исчерпан / юзер продолжил к ревью).
/// `Completed` — тренировка закрыта: одноэкранный переход «теперь к
/// повторению» (H-итерация: без него смена контекста непонятна юзеру).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AcquaintanceStage {
    #[default]
    Inactive,
    Presentation,
    Training,
    Completed,
}

/// UI-состояние руки: доменная машина (`AcquaintanceHand`) хранит правила,
/// этот тип — только позицию юзера внутри потока и служебные флаги.
#[derive(Clone, Default)]
pub struct AcquaintanceState {
    pub stage: AcquaintanceStage,
    pub hand: Option<AcquaintanceHand>,
    pub slide_index: usize,
    pub skipped_ids: HashSet<Ulid>,
    /// Рука закрывается: персистенция идёт, экран завершения ещё не
    /// смонтирован (stage станет Completed после коммита записи — защита
    /// бага #462). UI в этом окне замораживает отвеченную карту и прячет
    /// кнопки: без флага после финального ответа пере-показывался вопрос
    /// «лишней картой», которую затем резко перекрывал экран завершения.
    pub hand_finishing: bool,
}

/// Автозвук слова (механизм урока, lesson_card.rs): звучит, когда TTS
/// доступен и звук урока не выключен. Единый предикат для показа и
/// Forward-фронта тренировки — гарды не дублируются.
pub fn should_autoplay_word_audio(is_muted: bool, speech_supported: bool) -> bool {
    speech_supported && !is_muted
}

/// Видимость кнопки озвучки в шапке руки: кнопка озвучивает японскую
/// строку текущей карты — доступна, только когда JP на экране.
/// `voiced = None` (кандзи/счётные — беззвучные карты) прячет кнопку
/// целиком. Word: Reverse-фронт показывает перевод, аудио-фронт Forward
/// прячет JP до раскрытия — кнопку не показывать (подсказка голосом);
/// на стороне ответа JP видна — кнопка возвращается. FrontAlwaysVisible
/// (грамматика): JP-строка фронта на экране в любой подфазе и на ответе —
/// кнопка видна всегда (в том числе при `subphase = None` — рука без слов,
/// грамматика проходит её насквозь; монета аудио-фронтов несловесным картам
/// не выпадает — инвариант `resolve_audio_front`, поэтому audio_front
/// фронтовидом не гейтится).
pub fn audio_button_visible(
    stage: AcquaintanceStage,
    subphase: Option<AcquaintanceSubphase>,
    audio_front: bool,
    showing_answer: bool,
    voiced: Option<VoicedKind>,
) -> bool {
    let Some(kind) = voiced else {
        return false;
    };
    match stage {
        // Показ: японская строка (слово/паттерн) всегда на экране.
        AcquaintanceStage::Presentation => true,
        AcquaintanceStage::Training => match kind {
            VoicedKind::FrontAlwaysVisible => true,
            VoicedKind::Word => match subphase {
                Some(AcquaintanceSubphase::Reverse) => showing_answer,
                Some(AcquaintanceSubphase::Forward) => !audio_front || showing_answer,
                // Рука без подфаз не содержит слов: ветка недостижима при
                // kind = Word (подфаза None только у рук без слов — см.
                // AcquaintanceHand::new), значение сохранено из прежней логики.
                None => true,
            },
        },
        AcquaintanceStage::Completed | AcquaintanceStage::Inactive => false,
    }
}

impl AcquaintanceState {
    /// Ре-инициализация под новую руку: reload урока переиспользует тот же
    /// state (компонент не размонтируется), поэтому сбрасывать надо КАЖДОЕ
    /// поле потока. Несброшенный `hand_finishing` прошлой руки ослеплял
    /// вторую руку сессии — кнопки показа/тренировки и клавиатура
    /// гейтились навсегда (ревью PR #498).
    pub fn start_new_hand(&mut self, hand: AcquaintanceHand) {
        self.stage = AcquaintanceStage::Presentation;
        self.hand = Some(hand);
        self.slide_index = 0;
        self.skipped_ids.clear();
        self.hand_finishing = false;
    }

    /// Переходит к следующей непомеченной «Уже знаю» карте.
    /// Возвращает `true`, если показ исчерпан (пора в следующую стадию).
    pub fn advance_presentation(&mut self) -> bool {
        let Some(hand) = &self.hand else {
            return true;
        };
        let order = hand.presentation_order();
        loop {
            self.slide_index += 1;
            if self.slide_index >= order.len() {
                return true;
            }
            if !self.skipped_ids.contains(&order[self.slide_index]) {
                return false;
            }
        }
    }
}

/// Данные слайда показа, разрешённые из StudyCard до рендера
/// (индекс вектора совпадает с `presentation_order` руки).
#[derive(Clone)]
pub enum AcquaintanceSlideData {
    Vocabulary {
        card_id: Ulid,
        word: String,
        pos_label: Option<String>,
        translations: Vec<String>,
    },
    Kanji {
        card_id: Ulid,
        kanji: String,
        name: String,
        radicals: Option<Vec<RadicalDisplay>>,
        example_words: Option<Vec<(String, String)>>,
        on_readings: Option<Vec<ReadingItem>>,
        kun_readings: Option<Vec<ReadingItem>>,
    },
    Grammar {
        card_id: Ulid,
        /// Правило грамматики: раскрывает «Подробнее» на ответе тренировки
        /// (полное explanation/how-to-form/examples/nuances).
        rule_id: Ulid,
        pattern: String,
        short_description: String,
        how_to_form: String,
        examples: String,
        explanation: String,
        nuances: String,
    },
    /// Счётный суффикс (issue #415): показ — знак + глосс + вся таблица
    /// чтений (нерегулярные подсвечены); тренировка — фронт знак, ответ
    /// глосс + таблица. Несловесная карта: единый критерий, без подфаз.
    Counter {
        card_id: Ulid,
        suffix: String,
        meaning: String,
        /// (число-лейбл, чтение, нерегулярная) — вся таблица реестра.
        table: Vec<(String, String, bool)>,
    },
}

impl AcquaintanceSlideData {
    pub fn card_id(&self) -> Ulid {
        match self {
            Self::Vocabulary { card_id, .. }
            | Self::Kanji { card_id, .. }
            | Self::Grammar { card_id, .. }
            | Self::Counter { card_id, .. } => *card_id,
        }
    }

    /// Японское слово слайда (для озвучки из шапки); несловесные карты —
    /// `None`.
    pub fn word(&self) -> Option<&str> {
        match self {
            Self::Vocabulary { word, .. } => Some(word),
            _ => None,
        }
    }

    /// Озвучка слайда на данном этапе руки: вид (управляет видимостью
    /// кнопки в тренировке) и японская строка. Единый аксессор — вид и
    /// текст согласованы структурно: кнопка не может остаться без текста
    /// или появиться у беззвучной карты.
    ///
    /// Vocabulary: слово на обоих этапах. Grammar: показ — паттерн
    /// (визуальный якорь слайда, паритет с обычным уроком, где вопрос
    /// грамматической карты = паттерн), тренировка — текст фронта
    /// (`grammar_training_front_text`, ровно то, что рендерится).
    /// Кандзи и счётный суффикс беззвучны (#808). Пустые данные
    /// деградируют в тишину: pattern построен с `unwrap_or_default`,
    /// examples бывают пустыми.
    pub fn audio(&self, stage: AcquaintanceStage) -> Option<(VoicedKind, String)> {
        match self {
            Self::Vocabulary { word, .. } => Some((VoicedKind::Word, word.clone())),
            Self::Grammar {
                pattern, examples, ..
            } => {
                let text = match stage {
                    AcquaintanceStage::Presentation => pattern.clone(),
                    _ => grammar_training_front_text(pattern, examples),
                };
                (!text.is_empty()).then_some((VoicedKind::FrontAlwaysVisible, text))
            },
            Self::Kanji { .. } | Self::Counter { .. } => None,
        }
    }

    /// Часть речи слайда слова — тег в шапке.
    pub fn pos_label(&self) -> Option<&str> {
        match self {
            Self::Vocabulary { pos_label, .. } => pos_label.as_deref(),
            _ => None,
        }
    }
}

/// Контекст префазы урока.
#[derive(Clone)]
pub struct AcquaintanceContext {
    pub repository: HybridUserRepository,
    pub state: RwSignal<AcquaintanceState>,
    pub slides: RwSignal<Vec<AcquaintanceSlideData>>,
    pub known_kanji: RwSignal<HashSet<char>>,
    pub native_language: RwSignal<NativeLanguage>,
    /// Текущая карта тренировки — отдельный сигнал: шапка читает его для
    /// тега типа карты, а запись при монтаже TrainingBody не перезапускает
    /// родительские Show (state.update зацикливала бы их перемонтирование).
    pub current_card: RwSignal<Option<Ulid>>,
    /// Раскрыт ли ответ тренировки — шапке нужен для видимости кнопки
    /// озвучки (Reverse-фронт прячет JP-сторону). Отдельный сигнал:
    /// шапка и TrainingBody делят его без чтений общего state, а дерево
    /// префазы стабильно (Memo-гейт content.rs не перемонтирует его).
    pub showing_answer: RwSignal<bool>,
    /// Бросок монеты аудио-фронта ТЕКУЩЕЙ карты тренировки: свежий на
    /// каждый показ (без заморозки — решение владельца). Пишет только
    /// TrainingBody (Effect монеты, безусловная запись на каждый показ),
    /// читают шапка (видимость кнопки озвучки) и клавиатурный предикат.
    /// `true` достижим только для слова в Forward-подфазе — инвариант
    /// держит `resolve_audio_front`.
    pub audio_front: RwSignal<bool>,
}

impl AcquaintanceContext {
    /// Завершение руки: сидирование первого ревью назавтра + списание лимита
    /// одной операцией (S2), затем сразу ревью урока — без итогового
    /// экрана. Вызывается из конца тренировки (`HandCompleted`) и из
    /// показа, когда выведены все карты.
    ///
    /// Персистенция выполняется ДО перевода стадии в `Completed`: кнопка
    /// «К повторению» и её Space-хендлер существуют только после того, как
    /// запись закоммитилась, поэтому навигация или перезагрузка страницы
    /// физически не могут обогнать сейв (баг #462: сидирование терялось,
    /// когда тест/пользователь перезагружал страницу сразу после
    /// завершения руки — fire-and-forget `spawn_local` не успевал).
    pub fn complete_hand(&self) {
        // Флаг финиша — синхронно, до персистенции: UI сразу замораживает
        // отвеченную карту и прячет кнопки (см. hand_finishing), не дожидаясь
        // коммита записи и монтирования экрана завершения.
        self.state.update(|state| state.hand_finishing = true);

        let ids = self.state.with(|state| {
            state
                .hand
                .as_ref()
                .map(|h| h.presentation_order())
                .unwrap_or_default()
        });
        // Карты руки входят в «Пройдено» урока: экран завершения
        // показывает руку + ревью, а не 0 после урока из одной руки
        // (юзер-репорт). В wasm-тестах AcquaintanceView монтируется
        // без LessonContext — начисление просто пропускается.
        if let Some(lesson_ctx) = use_context::<super::lesson_state::LessonContext>() {
            lesson_ctx
                .lesson_state
                .update(|state| state.review_count += ids.len());
        }
        let repo = self.repository.clone();
        let state = self.state;
        // Unscoped on purpose: the persistence below must survive the
        // user leaving the lesson mid-save (#462). Reactive safety: the
        // task never reads signals after its await — the only signal
        // access is the final `update`, and writes to a disposed signal
        // are silent no-ops in reactive_graph.
        spawn_local(async move {
            // Сидирование выполняет CompleteAcquaintanceHandUseCase (S2):
            // первый ревью назавтра всем картам руки + лимит одной операцией.
            if let Err(e) = CompleteAcquaintanceHandUseCase::new(&repo)
                .execute(ids)
                .await
            {
                // Локальная запись (IndexedDB) падает крайне редко; застрять
                // на тренировке хуже, чем потерять сидирование — завершаем
                // и при ошибке, деградация видна в логах.
                tracing::error!("Acquaintance hand completion failed: {e}");
            }
            state.update(|state| state.stage = AcquaintanceStage::Completed);
        });
    }
}

#[cfg(test)]
mod audio_button_visible_tests {
    use super::*;

    const WORD: Option<VoicedKind> = Some(VoicedKind::Word);
    const GRAMMAR: Option<VoicedKind> = Some(VoicedKind::FrontAlwaysVisible);
    const SILENT: Option<VoicedKind> = None;

    #[rstest::rstest]
    #[case::presentation(true, None, false, false)]
    #[case::presentation_answer(true, None, false, true)]
    #[case::forward_text_front(true, Some(AcquaintanceSubphase::Forward), false, false)]
    #[case::forward_text_answer(true, Some(AcquaintanceSubphase::Forward), false, true)]
    #[case::forward_audio_front_hidden(false, Some(AcquaintanceSubphase::Forward), true, false)]
    #[case::forward_audio_answer_shows_button(
        true,
        Some(AcquaintanceSubphase::Forward),
        true,
        true
    )]
    #[case::reverse_front_hidden(false, Some(AcquaintanceSubphase::Reverse), false, false)]
    #[case::reverse_answer(true, Some(AcquaintanceSubphase::Reverse), false, true)]
    #[case::reverse_ignores_audio_front(false, Some(AcquaintanceSubphase::Reverse), true, false)]
    fn training_visibility_depends_on_jp_side(
        #[case] expected: bool,
        #[case] subphase: Option<AcquaintanceSubphase>,
        #[case] audio_front: bool,
        #[case] showing_answer: bool,
    ) {
        assert_eq!(
            audio_button_visible(
                AcquaintanceStage::Training,
                subphase,
                audio_front,
                showing_answer,
                WORD,
            ),
            expected
        );
    }

    /// Грамматика: JP-строка фронта (пример/паттерн) на экране в любой
    /// подфазе и на ответе — кнопка видна всегда. Рука без слов (только
    /// кандзи/грамматика) проходит тренировку с subphase = None — тот же
    /// «всегда видна». Монета аудио-фронтов несловесным картам не
    /// выпадает (инвариант resolve_audio_front), поэтому audio_front не
    /// гейтит вид.
    #[rstest::rstest]
    #[case::forward_front(true, Some(AcquaintanceSubphase::Forward), false, false)]
    #[case::forward_answer(true, Some(AcquaintanceSubphase::Forward), false, true)]
    #[case::reverse_front(true, Some(AcquaintanceSubphase::Reverse), false, false)]
    #[case::reverse_answer(true, Some(AcquaintanceSubphase::Reverse), false, true)]
    #[case::wordless_hand_no_subphase(true, None, false, false)]
    #[case::audio_front_flag_ignored(true, Some(AcquaintanceSubphase::Forward), true, false)]
    fn grammar_front_is_always_visible(
        #[case] expected: bool,
        #[case] subphase: Option<AcquaintanceSubphase>,
        #[case] audio_front: bool,
        #[case] showing_answer: bool,
    ) {
        assert_eq!(
            audio_button_visible(
                AcquaintanceStage::Training,
                subphase,
                audio_front,
                showing_answer,
                GRAMMAR,
            ),
            expected
        );
    }

    #[rstest::rstest]
    #[case::presentation_word(AcquaintanceStage::Presentation, WORD, true)]
    #[case::presentation_silent(AcquaintanceStage::Presentation, SILENT, false)]
    #[case::inactive_word(AcquaintanceStage::Inactive, WORD, false)]
    #[case::completed_word(AcquaintanceStage::Completed, WORD, false)]
    #[case::completed_grammar(AcquaintanceStage::Completed, GRAMMAR, false)]
    fn non_training_stages(
        #[case] stage: AcquaintanceStage,
        #[case] voiced: Option<VoicedKind>,
        #[case] expected: bool,
    ) {
        assert_eq!(
            audio_button_visible(stage, None, false, false, voiced),
            expected
        );
    }
}

#[cfg(test)]
mod slide_audio_tests {
    use super::*;

    fn grammar_slide(pattern: &str, examples: &str) -> AcquaintanceSlideData {
        AcquaintanceSlideData::Grammar {
            card_id: Ulid::new(),
            rule_id: Ulid::new(),
            pattern: pattern.to_string(),
            short_description: String::new(),
            how_to_form: String::new(),
            examples: examples.to_string(),
            explanation: String::new(),
            nuances: String::new(),
        }
    }

    fn vocab_slide(word: &str) -> AcquaintanceSlideData {
        AcquaintanceSlideData::Vocabulary {
            card_id: Ulid::new(),
            word: word.to_string(),
            pos_label: None,
            translations: vec!["читать".to_string()],
        }
    }

    /// Вид и текст согласованы структурно (единый аксессор): матрица
    /// фиксирует пару (kind, text) для каждого слайда и этапа.
    #[rstest::rstest]
    #[case::vocab_presentation(
        AcquaintanceStage::Presentation,
        Some((VoicedKind::Word, "読む".to_string()))
    )]
    #[case::vocab_training(
        AcquaintanceStage::Training,
        Some((VoicedKind::Word, "読む".to_string()))
    )]
    fn vocabulary_sounds_the_word_on_both_stages(
        #[case] stage: AcquaintanceStage,
        #[case] expected: Option<(VoicedKind, String)>,
    ) {
        // Arrange / Act / Assert
        assert_eq!(vocab_slide("読む").audio(stage), expected);
    }

    #[test]
    fn grammar_presentation_sounds_the_pattern() {
        // Arrange / Act / Assert
        assert_eq!(
            grammar_slide("～は～です", "```\n私は学生です。\nI am a student.\n```")
                .audio(AcquaintanceStage::Presentation),
            Some((VoicedKind::FrontAlwaysVisible, "～は～です".to_string()))
        );
    }

    #[test]
    fn grammar_training_sounds_the_rendered_front_text() {
        // Arrange: фронт тренировки — JP-строка первого примера без
        // emphasis-маркеров; озвучка обязана совпадать с рендером
        // Act / Assert
        assert_eq!(
            grammar_slide(
                "～は～です",
                "```\n彼は**学生**です。\nHe is a student.\n```"
            )
            .audio(AcquaintanceStage::Training),
            Some((VoicedKind::FrontAlwaysVisible, "彼は学生です。".to_string()))
        );
    }

    #[test]
    fn grammar_training_without_examples_falls_back_to_pattern() {
        // Arrange / Act / Assert
        assert_eq!(
            grammar_slide("～たことがある", "").audio(AcquaintanceStage::Training),
            Some((VoicedKind::FrontAlwaysVisible, "～たことがある".to_string()))
        );
    }

    #[rstest::rstest]
    #[case::empty_pattern_presentation(AcquaintanceStage::Presentation)]
    #[case::empty_everything_training(AcquaintanceStage::Training)]
    fn grammar_with_no_speakable_text_is_silent(#[case] stage: AcquaintanceStage) {
        // Arrange: pattern построен с unwrap_or_default, examples бывают
        // пустыми — пустая строка деградирует в тишину
        // Act / Assert
        assert_eq!(grammar_slide("", "").audio(stage), None);
    }

    /// Кандзи и счётный суффикс беззвучны на обоих этапах (#808): у
    /// голого знака нет единственного чтения.
    #[rstest::rstest]
    #[case::kanji_presentation(kanji_slide(), AcquaintanceStage::Presentation)]
    #[case::kanji_training(kanji_slide(), AcquaintanceStage::Training)]
    #[case::counter_presentation(counter_slide(), AcquaintanceStage::Presentation)]
    #[case::counter_training(counter_slide(), AcquaintanceStage::Training)]
    fn non_word_cards_are_silent(
        #[case] slide: AcquaintanceSlideData,
        #[case] stage: AcquaintanceStage,
    ) {
        assert_eq!(slide.audio(stage), None);
    }

    fn kanji_slide() -> AcquaintanceSlideData {
        AcquaintanceSlideData::Kanji {
            card_id: Ulid::new(),
            kanji: "読".to_string(),
            name: "читать".to_string(),
            radicals: None,
            example_words: None,
            on_readings: None,
            kun_readings: None,
        }
    }

    fn counter_slide() -> AcquaintanceSlideData {
        AcquaintanceSlideData::Counter {
            card_id: Ulid::new(),
            suffix: "枚".to_string(),
            meaning: "плоские предметы".to_string(),
            table: Vec::new(),
        }
    }
}

#[cfg(test)]
mod should_autoplay_word_audio_tests {
    use super::*;

    #[rstest::rstest]
    #[case::muted(true, true, false)]
    #[case::no_tts(false, false, false)]
    #[case::muted_and_no_tts(true, false, false)]
    #[case::ready(false, true, true)]
    fn autoplay_requires_tts_and_unmuted_lesson(
        #[case] is_muted: bool,
        #[case] speech_supported: bool,
        #[case] expected: bool,
    ) {
        assert_eq!(
            should_autoplay_word_audio(is_muted, speech_supported),
            expected
        );
    }
}

#[cfg(test)]
mod advance_presentation_tests {
    use super::*;
    use origa::domain::CardType;
    use std::collections::HashSet;

    fn state_with_hand(count: usize) -> AcquaintanceState {
        let pairs: Vec<(Ulid, CardType)> = (0..count)
            .map(|_| (Ulid::new(), CardType::Vocabulary))
            .collect();
        let hand = AcquaintanceHand::new(pairs).unwrap();
        AcquaintanceState {
            stage: AcquaintanceStage::Presentation,
            hand: Some(hand),
            slide_index: 0,
            skipped_ids: HashSet::new(),
            hand_finishing: false,
        }
    }

    #[test]
    fn advance_moves_through_slides_then_reports_exhausted() {
        // Arrange
        let mut state = state_with_hand(2);

        // Act / Assert
        assert!(!state.advance_presentation());
        assert_eq!(state.slide_index, 1);
        assert!(state.advance_presentation(), "показ исчерпан");
    }

    /// Reload урока переиспользует тот же state: несброшенное окно финиша
    /// прошлой руки ослепляло вторую руку сессии (кнопки/клавиатура
    /// гейтились навсегда) — ревью PR #498.
    #[test]
    fn start_new_hand_resets_finishing_window_and_stream_fields() {
        // Arrange: state закрытой руки — финиш, продвинутый показ, метки
        let mut state = state_with_hand(2);
        state.stage = AcquaintanceStage::Completed;
        state.slide_index = 1;
        state.hand_finishing = true;
        state.skipped_ids.insert(Ulid::new());

        // Act: reload пишет новую руку в тот же state
        let fresh = state_with_hand(1).hand.take().expect("hand built");
        state.start_new_hand(fresh);

        // Assert
        assert_eq!(state.stage, AcquaintanceStage::Presentation);
        assert_eq!(state.slide_index, 0);
        assert!(
            !state.hand_finishing,
            "окно финиша прошлой руки снято — кнопки новой руки работают"
        );
        assert!(state.skipped_ids.is_empty());
    }

    #[test]
    fn advance_skips_known_marked_cards() {
        // Arrange: средняя карта помечена «Уже знаю»
        let [a, b, c] = [Ulid::new(), Ulid::new(), Ulid::new()];
        let pairs = vec![
            (a, CardType::Vocabulary),
            (b, CardType::Vocabulary),
            (c, CardType::Vocabulary),
        ];
        let hand = AcquaintanceHand::new(pairs).unwrap();
        let mut state = AcquaintanceState {
            stage: AcquaintanceStage::Presentation,
            hand: Some(hand),
            slide_index: 0,
            skipped_ids: HashSet::from([b]),
            hand_finishing: false,
        };

        // Act / Assert: первый advance перепрыгивает b и показывает c
        assert!(!state.advance_presentation());
        assert_eq!(state.slide_index, 2);
        // следующий advance исчерпывает показ
        assert!(state.advance_presentation());
    }
}

#[cfg(test)]
mod front_signal_tests {
    use super::*;

    /// Матрица инварианта: аудио-фронт достижим ТОЛЬКО для слова в
    /// Forward-подфазе при выпавшей монете и реальной озвучиваемости —
    /// на Reverse и у несловесных карт фронт текстовый независимо от
    /// броска (иначе протухший аудио-фронт озвучивал бы ответ Reverse
    /// или ломал Space на кандзи/грамматике).
    #[rstest::rstest]
    #[case::word_forward_roll_available(
        true,
        Some(AcquaintanceSubphase::Forward),
        true,
        true,
        true
    )]
    #[case::word_forward_roll_unavailable(
        true,
        Some(AcquaintanceSubphase::Forward),
        true,
        false,
        false
    )]
    #[case::word_forward_no_roll(true, Some(AcquaintanceSubphase::Forward), false, true, false)]
    #[case::word_reverse_never(true, Some(AcquaintanceSubphase::Reverse), true, true, false)]
    #[case::word_reverse_no_roll_never(
        true,
        Some(AcquaintanceSubphase::Reverse),
        false,
        true,
        false
    )]
    #[case::non_word_forward_never(false, Some(AcquaintanceSubphase::Forward), true, true, false)]
    #[case::word_no_subphase_never(true, None, true, true, false)]
    fn audio_front_only_for_words_in_forward(
        #[case] is_word: bool,
        #[case] subphase: Option<AcquaintanceSubphase>,
        #[case] roll: bool,
        #[case] audio_available: bool,
        #[case] expected: bool,
    ) {
        assert_eq!(
            resolve_audio_front(is_word, subphase, roll, audio_available),
            expected
        );
    }

    #[test]
    fn start_new_hand_resets_presentation_stream() {
        let card_id = Ulid::new();
        let mut state = AcquaintanceState {
            stage: AcquaintanceStage::Completed,
            slide_index: 1,
            hand_finishing: true,
            skipped_ids: HashSet::from([card_id]),
            ..AcquaintanceState::default()
        };

        let hand =
            AcquaintanceHand::new(vec![(card_id, origa::domain::CardType::Vocabulary)]).unwrap();
        state.start_new_hand(hand);

        assert_eq!(state.stage, AcquaintanceStage::Presentation);
        assert_eq!(state.slide_index, 0);
        assert!(!state.hand_finishing);
        assert!(state.skipped_ids.is_empty());
    }
}
