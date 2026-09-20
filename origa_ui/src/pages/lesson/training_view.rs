use super::acquaintance_keyboard::{
    AcquaintanceKeyboardActions, create_acquaintance_keyboard_handler,
};
use super::acquaintance_state::{
    AcquaintanceContext, AcquaintanceSlideData, resolve_audio_front, should_autoplay_word_audio,
};
use super::keyboard_handler::is_typing_target;
use super::training_answer::TrainingAnswerSlide;
use super::training_front::TrainingFrontSlide;
use crate::i18n::*;
use crate::ui_components::{
    Button, ButtonVariant, is_speech_supported, speak_word, word_audio_available,
};
use leptos::prelude::*;
use leptos_use::use_event_listener;
use origa::domain::{AcquaintanceSubphase, AnswerOutcome};
use ulid::Ulid;

/// Раскрытие ответа: и кнопка, и Space ведут себя одинаково.
/// Окно финиша руки (hand_finishing) игнорируется: карта уже отвечена,
/// повторное раскрытие невозможно.
fn do_reveal(
    ctx_stored: &StoredValue<AcquaintanceContext>,
    current_id: &Memo<Ulid>,
    showing_answer: &RwSignal<bool>,
    muted: bool,
) {
    let ctx = ctx_stored.get_value();
    if ctx.state.with_untracked(|state| state.hand_finishing) {
        return;
    }
    showing_answer.set(true);
    let card_id = current_id.get_untracked();
    // Автозвук ответа Reverse подчиняется тому же предикату, что и автозвук
    // слова: мьют урока глушит его (баг-репорт: в мьют-режиме ответ
    // озвучивался). Повтор остаётся доступен кнопкой озвучки в шапке.
    if !card_id.is_nil()
        && super::acquaintance_state::should_autoplay_word_audio(muted, is_speech_supported())
    {
        speak_reverse_answer(&ctx, card_id);
    }
}

/// Запись ответа: и кнопки [1]/[2], и клавиши 1/2 ведут себя одинаково.
/// Окно финиша руки (hand_finishing) игнорируется — защита от двойного
/// клика/клавиши до монтирования экрана завершения.
fn do_rate(
    ctx_stored: &StoredValue<AcquaintanceContext>,
    current_id: &Memo<Ulid>,
    showing_answer: &RwSignal<bool>,
    rotation_index: &RwSignal<usize>,
    training_order: &RwSignal<Vec<Ulid>>,
    remembered: bool,
) {
    let ctx = ctx_stored.get_value();
    if ctx.state.with_untracked(|state| state.hand_finishing) {
        return;
    }
    let outcome = record_on_hand(&ctx, current_id.get_untracked(), remembered);
    finish_answer(
        &ctx,
        showing_answer,
        rotation_index,
        training_order,
        outcome,
    );
    // Шапке нужен тип новой текущей карты (тег типа).
    ctx.current_card.set(current_id.get_untracked().non_nil());
}

/// Fisher–Yates поверх xorshift64; источник энтропии — случайные биты
/// новых Ulid (крейт уже в дереве), внешних зависимостей не добавляет.
fn shuffled_order(mut cards: Vec<Ulid>) -> Vec<Ulid> {
    let mut seed = {
        let a = Ulid::new().0;
        let b = Ulid::new().0;
        (a ^ (b >> 1)) as u64 | 1
    };
    for i in (1..cards.len()).rev() {
        seed ^= seed << 13;
        seed ^= seed >> 7;
        seed ^= seed << 17;
        let j = (seed as usize) % (i + 1);
        cards.swap(i, j);
    }
    cards
}

/// Кандидаты витка тренировки: активные карты руки; в Reverse-подфазе —
/// только слова. K-итерация: последняя фаза РУ→ЯП не показывает кандзи и
/// грамматику — их критерий закрыт до смены подфазы (см.
/// `AcquaintanceHand::advance_subphase_if_words_done`), словесной ротации
/// они не касаются.
fn rotation_candidates(hand: &origa::domain::AcquaintanceHand) -> Vec<Ulid> {
    let words_only = hand.subphase() == Some(AcquaintanceSubphase::Reverse);
    hand.presentation_order()
        .into_iter()
        .filter(|id| {
            hand.entry(*id).is_some_and(|entry| {
                !entry.is_retired()
                    && (!words_only || entry.card_type() == origa::domain::CardType::Vocabulary)
            })
        })
        .collect()
}

/// Витковый порядок тренировки: presentation_order минус выведенные карты,
/// перемешанный на каждый виток и каждую смену подфазы.
fn build_rotation_order(ctx: &AcquaintanceContext) -> Vec<Ulid> {
    // with_untracked: чтение при создании компонента вне реактивного
    // контекста не должно подписываться (и не плодит console-warning).
    ctx.state.with_untracked(|state| {
        let Some(hand) = state.hand.as_ref() else {
            return Vec::new();
        };
        shuffled_order(rotation_candidates(hand))
    })
}

/// Тренировка (S5): полные ротации руки до критерия каждой карты.
/// Порядок витка перемешивается заново на каждый виток и смену подфазы;
/// закрывшие критерий продолжают отвечаться: успехи заморожены, «Не помню»
/// переоткрывает карту (шкала подфазы — в ноль).
///
/// Единственный источник истины для перерисовки — `AnswerOutcome`
/// из доменной машины; UI не дублирует подсчёт успехов.
#[component]
pub fn TrainingBody(ctx: AcquaintanceContext) -> impl IntoView {
    let i18n = use_i18n();
    let ctx_stored = StoredValue::new(ctx);
    // Раскрытие ответа живёт в контексте: шапке он нужен для видимости
    // кнопки озвучки (JP-сторона скрыта на Reverse-фронте).
    let showing_answer = ctx_stored.get_value().showing_answer;
    showing_answer.set(false);
    // Монета аудио-фронта — тоже контекстный сигнал: сброс при монтаже,
    // дальше её пишет только Effect монеты (по записи на каждый показ).
    ctx_stored.get_value().audio_front.set(false);
    let rotation_index = RwSignal::new(0usize);
    let training_order = RwSignal::new(build_rotation_order(&ctx_stored.get_value()));

    let current_id = Memo::new(move |_| {
        let order = training_order.get();
        if order.is_empty() {
            return Ulid::nil();
        }
        order[rotation_index.get() % order.len()]
    });

    // Mute + pitch-loader readiness for the audio-front availability
    // sample (same predicate inputs as the lesson AudioRecall mode).
    let lesson_ctx = use_context::<super::lesson_state::LessonContext>();
    let is_muted = lesson_ctx.as_ref().map(|ctx| ctx.is_muted);
    let pitch_ready = use_context::<crate::store::auth_store::AuthStore>()
        .map(|store| store.is_pitch_audio_loaded);

    // Эффект броска: РОВНО ОДНА безусловная запись сигнала на запуск —
    // значение только из resolve_audio_front, ранних выходов без записи
    // нет. Иначе протухший `true` с аудио-фронта слова доживал бы до
    // следующей несловесной карты (Space вместо «Показать») или до
    // Reverse-подфазы (озвучка ответа на текстовом фронте).
    // Триггеры переролла — TRACKED current_id + rotation_index: в
    // однословной руке card_id между витками не меняется, а
    // rotation_index растёт на каждом ответе (смена подфазы тоже —
    // SwitchedSubphase делает set(0)). Подфаза, мьют и озвучиваемость
    // читаются UNTRACKED: записи ответов и окно финиша монету не
    // перебрасывают — фронт стабилен в пределах показа.
    let coin_ctx = ctx_stored;
    Effect::new(move |_| {
        let card_id = current_id.get();
        // TRACKED-чтение без использования значения: rotation_index —
        // второй триггер переролла (однословная рука, смена подфазы).
        let _rotation = rotation_index.get();
        let ctx = coin_ctx.get_value();
        let decision = ctx.state.with_untracked(|state| {
            let subphase = state.hand.as_ref().and_then(|h| h.subphase());
            let word = ctx.slides.with_untracked(|slides| {
                slides
                    .iter()
                    .find(|slide| slide.card_id() == card_id)
                    .and_then(|slide| slide.word().map(str::to_string))
            });
            let muted = is_muted
                .as_ref()
                .and_then(|signal| signal.try_get_untracked())
                .unwrap_or(false);
            let pitch = pitch_ready
                .as_ref()
                .and_then(|signal| signal.try_get_untracked())
                .unwrap_or(false);
            let audio_available =
                !muted && pitch && word.as_deref().is_some_and(word_audio_available);
            // Случайность — младший бит свежего Ulid (крейт уже в дереве).
            let roll = ulid::Ulid::new().0 & 1 == 0;
            resolve_audio_front(word.is_some(), subphase, roll, audio_available)
        });
        ctx.audio_front.set(decision);
    });
    // Автозвук фронта — один на СМЕНУ карты: Memo по card_id
    // молчит при перезапусках рендер-замыкания с той же картой (запись
    // ответа, пересборка slides), звучит только новой карте. Озвучивается
    // любой фронт Forward-подфазы: текстовый (слово на экране) и
    // аудио-фронт (слово только звучит) — одинаково через speak_word,
    // монета на автозвук не влияет. Reverse-фронт не звучит: слово
    // прозвучит при раскрытии ответа (speak_reverse_answer).
    // Тот же канал, что и автозвук обычного урока (speak_word): сначала
    // CDN pitch-аудио файла, TTS — только fallback (юзер-репорт: в
    // знакомстве звучал TTS вместо аудиофайла).
    let autoplay_ctx = ctx_stored;
    Effect::new(move |_| {
        let card_id = current_id.get();
        if card_id.is_nil() {
            return;
        }
        let ctx = autoplay_ctx.get_value();
        let reverse = ctx
            .state
            .with_untracked(|state| state.hand.as_ref().and_then(|h| h.subphase()))
            == Some(AcquaintanceSubphase::Reverse);
        if reverse {
            return;
        }
        let word = ctx
            .slides
            .get_untracked()
            .iter()
            .find(|slide| slide.card_id() == card_id)
            .and_then(|slide| slide.word().map(str::to_string));
        let Some(word) = word else {
            return;
        };
        let muted = is_muted
            .as_ref()
            .map(|signal| signal.get_untracked())
            .unwrap_or(false);
        // Availability = pitch audio OR TTS (not TTS-only): in pitch-only
        // environments the audio front must still autoplay.
        if !should_autoplay_word_audio(muted, word_audio_available(&word)) {
            return;
        }
        speak_word(&word, 1.0);
    });

    // Текущая карта тренировки для тега типа в шапке: отдельный сигнал,
    // запись при монтаже не перезапускает родительские Show.
    ctx_stored
        .get_value()
        .current_card
        .set(current_id.get_untracked().non_nil());

    // Клавиатура: те же хендлы, что у кнопок (спека §8.3). Гейты окна
    // финиша — внутри do_reveal/do_rate.
    let muted_now = move || {
        is_muted
            .as_ref()
            .map(|signal| signal.get_untracked())
            .unwrap_or(false)
    };
    let handle_keydown = {
        let c = ctx_stored;
        let audio_front_probe = ctx_stored.get_value().audio_front;
        let muted_for_keys = muted_now;
        create_acquaintance_keyboard_handler(
            c.get_value(),
            showing_answer,
            AcquaintanceKeyboardActions {
                // Advance разрешён только в показе; в тренировке — Reveal/Rate.
                on_advance: Box::new(|| {}),
                on_reveal: Box::new(move || {
                    do_reveal(&c, &current_id, &showing_answer, muted_for_keys());
                }),
                on_rate: Box::new(move |remembered: bool| {
                    do_rate(
                        &c,
                        &current_id,
                        &showing_answer,
                        &rotation_index,
                        &training_order,
                        remembered,
                    );
                }),
                on_replay_audio: Box::new(move || {
                    // Явное действие пользователя: мьют не гейтит повтор
                    // (безтекстовый фронт без звука нерешаем).
                    let card_id = current_id.get_untracked();
                    if card_id.is_nil() {
                        return;
                    }
                    let ctx = c.get_value();
                    if let Some(word) = ctx
                        .slides
                        .get_untracked()
                        .iter()
                        .find(|slide| slide.card_id() == card_id)
                        .and_then(|slide| slide.word().map(str::to_string))
                    {
                        speak_word(&word, 1.0);
                    }
                }),
            },
            Box::new(move || audio_front_probe.get_untracked()),
        )
    };

    let _ = use_event_listener(document(), leptos::ev::keydown, move |ev| {
        if is_typing_target(ev.target().as_ref()) {
            return;
        }
        handle_keydown(ev);
    });

    // Окно финиша: кнопки ухода со слайда скрыты, пока рука закрывается.
    let hand_finishing = Signal::derive(move || {
        let ctx = ctx_stored.get_value();
        ctx.state.with(|state| state.hand_finishing)
    });

    view! {
        <div
            class="flex flex-col grow"
            data-testid="acquaintance-training"
            data-card-id=move || {
                let id = current_id.get();
                if id.is_nil() {
                    String::new()
                } else {
                    id.to_string()
                }
            }
        >
            <div class="flex-1 py-2 sm:py-3">
                {move || {
                    let Some(card_id) = current_id.get().non_nil() else {
                        return ().into_any();
                    };
                    let reverse = ctx_stored.get_value().state.with(|state| {
                        state.hand.as_ref().and_then(|h| h.subphase())
                    }) == Some(AcquaintanceSubphase::Reverse);
                    // Приглушение фронта на стороне ответа — только для слов
                    // и кандзи. Фронт грамматики — японский пример, который
                    // и есть носитель правила: мут/прозрачность сверху
                    // мешают сверяться с ним, пока внизу раскрыт смысл
                    // (юзер-репорт). Приглушение — семантический класс
                    // front-dimmed, не tailwind opacity-60: голая opacity
                    // каскадировала в тултип кандзи и делала его
                    // полупрозрачным.
                    let front_is_grammar = ctx_stored
                        .get_value()
                        .slides
                        .get_untracked()
                        .iter()
                        .any(|slide| {
                            slide.card_id() == card_id
                                && matches!(slide, AcquaintanceSlideData::Grammar { .. })
                        });
                    view! {
                        <div
                            class=move || {
                                if showing_answer.get() && !front_is_grammar {
                                    "pt-1 pb-2 scale-90 origin-top front-dimmed"
                                } else {
                                    ""
                                }
                            }
                        >
                            <TrainingFrontSlide
                                ctx=ctx_stored.get_value()
                                card_id=card_id
                                reverse=reverse
                                audio_front=ctx_stored.get_value().audio_front.get()
                            />
                        </div>
                        <Show when=move || showing_answer.get() fallback=move || ()>
                            <div
                                class="border-t border-[var(--border-light)] my-2"
                                data-testid="acquaintance-answer-divider"
                            ></div>
                            <TrainingAnswerSlide
                                ctx=ctx_stored.get_value()
                                card_id=card_id
                                reverse=reverse
                            />
                        </Show>
                    }
                    .into_any()
                }}
            </div>
            <Show when=move || !showing_answer.get() && !hand_finishing.get() fallback=move || ()>
                <div class="flex justify-center">
                    <Button
                        variant=Signal::derive(|| ButtonVariant::Filled)
                        on_click=Callback::new(move |_| {
                            do_reveal(&ctx_stored, &current_id, &showing_answer, muted_now());
                        })
                        test_id=Signal::derive(|| "acquaintance-reveal-btn".to_string())
                    >
                        {t!(i18n, lesson.show_answer)}
                        // Space = «Показать ответ» на любом фронте
                        // (единый паттерн урока); повтор аудио — Enter.
                        <span class="kbd-hint">{t!(i18n, lesson.space_key)}</span>
                    </Button>
                </div>
            </Show>
            <Show when=move || showing_answer.get() && !hand_finishing.get() fallback=move || ()>
                <div class="mt-4 grid grid-cols-2 gap-3">
                    <Button
                        variant=Signal::derive(|| ButtonVariant::Default)
                        on_click=Callback::new(move |_: leptos::ev::MouseEvent| {
                            do_rate(
                                &ctx_stored,
                                &current_id,
                                &showing_answer,
                                &rotation_index,
                                &training_order,
                                false,
                            );
                        })
                        test_id=Signal::derive(|| "acquaintance-rating-dont-know".to_string())
                    >
                        {t!(i18n, acquaintance.dont_remember)}
                        <span class="kbd-hint">"[1]"</span>
                    </Button>
                    <Button
                        variant=Signal::derive(|| ButtonVariant::Olive)
                        on_click=Callback::new(move |_: leptos::ev::MouseEvent| {
                            do_rate(
                                &ctx_stored,
                                &current_id,
                                &showing_answer,
                                &rotation_index,
                                &training_order,
                                true,
                            );
                        })
                        test_id=Signal::derive(|| "acquaintance-rating-remember".to_string())
                    >
                        {t!(i18n, acquaintance.remember)}
                        <span class="kbd-hint">"[2]"</span>
                    </Button>
                </div>
            </Show>
        </div>
    }
}

trait NonNilUlid {
    fn non_nil(self) -> Option<Ulid>;
}

impl NonNilUlid for Ulid {
    fn non_nil(self) -> Option<Ulid> {
        (!self.is_nil()).then_some(self)
    }
}

/// Повтор аудио в ответе Reverse-подфазы (спека §8.2); остальные случаи —
/// фронт Forward уже прозвучал при показе слова в презентации.
fn speak_reverse_answer(ctx: &AcquaintanceContext, card_id: Ulid) {
    let reverse = ctx
        .state
        .with(|state| state.hand.as_ref().and_then(|h| h.subphase()))
        == Some(AcquaintanceSubphase::Reverse);
    if !reverse {
        return;
    }
    if let Some(AcquaintanceSlideData::Vocabulary { word, .. }) =
        ctx.slides.get().iter().find(|s| s.card_id() == card_id)
    {
        speak_if_supported(word);
    }
}

fn record_on_hand(ctx: &AcquaintanceContext, card_id: Ulid, remembered: bool) -> AnswerOutcome {
    let mut outcome = AnswerOutcome::ProgressFrozen;
    ctx.state.update(|state| {
        outcome = match state.hand.as_mut() {
            Some(hand) => hand.record_answer(card_id, remembered).unwrap_or_else(|e| {
                tracing::error!("Record answer failed for {card_id}: {e}");
                AnswerOutcome::ProgressFrozen
            }),
            None => AnswerOutcome::ProgressFrozen,
        };
    });
    outcome
}

/// Граница витка тренировки: что делать после очередного ответа.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Действие после ответа тренировки (docs/acquaintance-mode.md,
/// правило «Тренировка», H-итерация UX):
/// - успех, закрывший forward-критерий последнему активному слову, меняет
///   сторону НЕМЕДЛЕННО: полоса заполнилась — юзер видит переход и
///   сброшенные шкалы, а не «полную полосу без перехода»;
/// - иначе — следующая карта круга; на границе круга порядок
///   перемешивается. Ошибка юзера порядок не трогает.
pub enum AfterAnswerAction {
    /// Все активные слова закрыли подфазу: новый круг с нуля (Reverse),
    /// порядок перемешивается.
    SwitchedSubphase,
    /// Следующая карта текущего круга; `reshuffled` — круг закончился и
    /// порядок нового круга перемешен.
    NextCard { reshuffled: bool },
}

pub fn after_answer(
    success: bool,
    rotation_index: usize,
    active_len: usize,
    hand: &mut origa::domain::AcquaintanceHand,
) -> AfterAnswerAction {
    if success && hand.advance_subphase_if_words_done() {
        return AfterAnswerAction::SwitchedSubphase;
    }
    if active_len == 0 || (rotation_index + 1) % active_len != 0 {
        return AfterAnswerAction::NextCard { reshuffled: false };
    }
    AfterAnswerAction::NextCard { reshuffled: true }
}

/// Перемешивание круга с защитой стыка: первая карта нового круга не
/// повторяет последнюю карту предыдущего — иначе одна карта идёт дважды
/// подряд и кажется, что круг «застрял».
fn reshuffle_avoiding_repeat(prev_last: Option<Ulid>, mut cards: Vec<Ulid>) -> Vec<Ulid> {
    if cards.len() > 1 {
        cards = shuffled_order(cards);
        if prev_last.is_some_and(|last| cards[0] == last) {
            cards.swap(0, 1);
        }
    }
    cards
}

fn finish_answer(
    ctx: &AcquaintanceContext,
    showing_answer: &RwSignal<bool>,
    rotation_index: &RwSignal<usize>,
    training_order: &RwSignal<Vec<Ulid>>,
    outcome: AnswerOutcome,
) {
    // Финальный ответ: рука закрывается. Отвеченная карта ОСТАЁТСЯ на
    // экране (showing_answer не сбрасывается) — без этого фронт того же
    // слова пере-показывался «лишним вопросом», пока идёт запись и до
    // монтирования экрана завершения (юзер-репорт о резком переходе).
    // Кнопки и клавиатура гасятся флагом hand_finishing.
    if matches!(outcome, AnswerOutcome::HandCompleted) {
        ctx.complete_hand();
        return;
    }

    showing_answer.set(false);

    let prev_last = training_order.get_untracked().last().copied();
    let mut action = AfterAnswerAction::NextCard { reshuffled: false };
    ctx.state.update(|state| {
        if let Some(hand) = state.hand.as_mut() {
            action = after_answer(
                matches!(outcome, AnswerOutcome::Counted { .. }),
                rotation_index.get_untracked(),
                training_order.get_untracked().len(),
                hand,
            );
        }
    });
    match action {
        AfterAnswerAction::SwitchedSubphase => {
            // Сторона сменилась в момент заполнения полосы: новый круг с
            // нуля в перемешанном порядке. Порядок ПЕРЕСТРАИВАЕТСЯ из
            // кандидатов новой подфазы (K-итерация: Reverse-витки — только
            // слова), иначе ротация тащила бы кандзи/грамматику из
            // Forward-порядка. Сброс rotation_index — триггер переролла
            // монеты: аудио-фронт в новой подфазе не достижим (Reverse
            // всегда текстовый), бросок будет свежим при возврате в
            // Forward следующей руки.
            rotation_index.set(0);
            let candidates = ctx.state.with_untracked(|state| {
                state
                    .hand
                    .as_ref()
                    .map(rotation_candidates)
                    .unwrap_or_default()
            });
            training_order.set(reshuffle_avoiding_repeat(prev_last, candidates));
        },
        AfterAnswerAction::NextCard { reshuffled } => {
            rotation_index.set(rotation_index.get_untracked() + 1);
            if reshuffled {
                training_order.set(reshuffle_avoiding_repeat(
                    prev_last,
                    training_order.get_untracked(),
                ));
            }
        },
    }
}

/// Повтор аудио слова в ответе Reverse-подфазы (спека §8.2); guard
/// is_speech_supported гасит среды без TTS.
fn speak_if_supported(word: &str) {
    if is_speech_supported() {
        speak_word(word, 1.0);
    }
}

#[cfg(test)]
mod rotation_tests {
    use super::*;
    use origa::domain::CardType;

    fn two_word_hand() -> (origa::domain::AcquaintanceHand, Ulid, Ulid) {
        let a = Ulid::new();
        let b = Ulid::new();
        let hand = origa::domain::AcquaintanceHand::new(vec![
            (a, CardType::Vocabulary),
            (b, CardType::Vocabulary),
        ])
        .unwrap();
        (hand, a, b)
    }

    /// K-итерация: Reverse-подфаза — только слова. Несловесные карты (кандзи,
    /// грамматика) в кандидатов последней фазы не попадают; retired-карты
    /// исключены из ротации в любой подфазе.
    #[rstest::rstest]
    #[case::reverse_excludes_kanji(CardType::Kanji)]
    #[case::reverse_excludes_grammar(CardType::Grammar)]
    fn rotation_candidates_in_reverse_keep_words_only(#[case] nonword_type: CardType) {
        // Arrange: слово + несловесная карта, обе закрывают критерий
        let word = Ulid::new();
        let nonword = Ulid::new();
        let mut hand = origa::domain::AcquaintanceHand::new(vec![
            (word, CardType::Vocabulary),
            (nonword, nonword_type),
        ])
        .unwrap();
        for _ in 0..3 {
            hand.record_answer(word, true).unwrap();
            hand.record_answer(nonword, true).unwrap();
        }

        // Act / Assert: Forward — обе активные карты
        let forward = rotation_candidates(&hand);
        assert!(forward.contains(&word) && forward.contains(&nonword));

        // Reverse — только слово (несловесные закрыты до смены, их в
        // последней фазе нет)
        assert!(hand.advance_subphase_if_words_done());
        let reverse = rotation_candidates(&hand);
        assert_eq!(reverse, vec![word], "Reverse-витки состоят только из слов");
    }

    #[test]
    fn rotation_candidates_exclude_retired_cards() {
        // Arrange: два слова, одно выведено
        let a = Ulid::new();
        let b = Ulid::new();
        let mut hand = origa::domain::AcquaintanceHand::new(vec![
            (a, CardType::Vocabulary),
            (b, CardType::Vocabulary),
        ])
        .unwrap();
        hand.retire_card(b);

        // Act
        let candidates = rotation_candidates(&hand);

        // Assert
        assert_eq!(candidates, vec![a], "retired карта не отвечает в ротации");
    }

    /// Полный цикл двух слов: пока не все закрыли forward — следующая
    /// карта круга; закрывающий успех меняет сторону немедленно, и шкалы
    /// слов сбрасываются (полоса больше не «висит заполненной»).
    #[test]
    fn closing_success_switches_subphase_immediately() {
        // Arrange: [a, b], все ответы «помню»
        let (mut hand, a, b) = two_word_hand();

        // Круги 1-2 (ответы с индексами 0..3): сторона та же.
        for answer_index in 0..4 {
            let action = after_answer(true, answer_index, 2, &mut hand);
            let at_boundary = (answer_index + 1) % 2 == 0;
            assert!(matches!(
                action,
                AfterAnswerAction::NextCard { reshuffled } if reshuffled == at_boundary
            ));
            hand.record_answer(if answer_index % 2 == 0 { a } else { b }, true)
                .unwrap();
        }

        // 5-й ответ (индекс 4): a закрыл forward, но b ещё нет — смены нет
        // (и это ещё не граница круга).
        hand.record_answer(a, true).unwrap();
        assert!(matches!(
            after_answer(true, 4, 2, &mut hand),
            AfterAnswerAction::NextCard { reshuffled: false }
        ));

        // 6-й ответ (индекс 5): b закрывает forward — смена НЕМЕДЛЕННО.
        hand.record_answer(b, true).unwrap();
        assert!(matches!(
            after_answer(true, 5, 2, &mut hand),
            AfterAnswerAction::SwitchedSubphase
        ));
        assert_eq!(
            hand.subphase(),
            Some(origa::domain::AcquaintanceSubphase::Reverse)
        );
    }

    /// Закрытие mid-круга (ошибка отодвинула слово) тоже меняет сторону
    /// сразу — юзер не доигрывает круг с «полной полосой».
    #[test]
    fn mid_circle_closing_answer_switches_without_waiting_boundary() {
        // Arrange: [a, b]; b ошибся на первом круге
        let (mut hand, a, b) = two_word_hand();
        hand.record_answer(a, true).unwrap();
        hand.record_answer(b, false).unwrap();
        hand.record_answer(a, true).unwrap();
        hand.record_answer(b, true).unwrap();
        hand.record_answer(a, true).unwrap(); // a закрыл (5-й, граница была на 4-м и 6-м)
        assert!(matches!(
            after_answer(true, 5, 2, &mut hand),
            AfterAnswerAction::NextCard { reshuffled: true }
        ));
        hand.record_answer(b, true).unwrap(); // b: 2/3 (7-й ответ, индекс 5)
        assert!(matches!(
            after_answer(true, 5, 2, &mut hand),
            AfterAnswerAction::NextCard { reshuffled: true }
        ));
        // 8-й ответ (индекс 6, середина круга): b закрывает forward —
        // смена сразу, не ждём границы.
        hand.record_answer(b, true).unwrap(); // b: 3/3
        assert!(matches!(
            after_answer(true, 6, 2, &mut hand),
            AfterAnswerAction::SwitchedSubphase
        ));
    }

    /// Ошибка не двигает смену: subphase остаётся, только карта круга.
    #[test]
    fn failed_answer_keeps_subphase_and_order() {
        let (mut hand, _a, _b) = two_word_hand();
        let outcome = hand.record_answer(_a, false).unwrap();
        assert!(matches!(outcome, AnswerOutcome::Failed));
        assert!(matches!(
            after_answer(false, 0, 2, &mut hand),
            AfterAnswerAction::NextCard { reshuffled: false }
        ));
        assert_eq!(
            hand.subphase(),
            Some(origa::domain::AcquaintanceSubphase::Forward)
        );
    }

    /// Стык кругов: первая карта нового круга не повторяет последнюю
    /// карту предыдущего (рука > 1 карты); одиночная карта неизбежно
    /// повторяется.
    #[test]
    fn reshuffle_avoiding_repeat_keeps_seam_distinct() {
        let x = Ulid::new();
        let y = Ulid::new();
        let z = Ulid::new();
        let cards = vec![y, z, x];
        let reshuffled = reshuffle_avoiding_repeat(Some(x), cards);
        assert_ne!(
            reshuffled[0], x,
            "первая карта нового круга не повторяет последнюю прошлого"
        );
        assert_eq!(reshuffled.len(), 3);

        // Одиночная карта: дубль неизбежен, порядок сохранён.
        let single = reshuffle_avoiding_repeat(Some(x), vec![x]);
        assert_eq!(single, vec![x]);
    }
}
