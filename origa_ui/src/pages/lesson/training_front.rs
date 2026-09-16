use super::acquaintance_state::{AcquaintanceContext, AcquaintanceSlideData};
use super::grammar_example::grammar_example_front;
use crate::i18n::*;
use crate::ui_components::{FuriganaText, TranslatorText, speak_word, strip_emphasis_markers};
use leptos::prelude::*;
use leptos_icons::Icon;
use origa::domain::NativeLanguage;
use std::collections::HashSet;
use ulid::Ulid;
/// Forward-фронт слова в тренировке: чистый рендер — автозвук живёт в
/// `TrainingBody` одним Effect'ом по Memo текущей карты (дедуп: один
/// звук на смену карты, а не на каждое перемонтирование фронта).
#[component]
fn WordTrainingFront(
    word: String,
    known_kanji: HashSet<char>,
    native_language: NativeLanguage,
) -> impl IntoView {
    view! {
        <p class="font-serif text-5xl text-[var(--fg-black)] break-words">
            <FuriganaText
                text=word
                known_kanji
                native_language=native_language
                with_kanji_tooltip=true
            />
        </p>
    }
}

/// Фронт тренировки: японская сторона (Forward; монета может показать
/// аудио-фронт — слово только звучит, иконка + повтор) или перевод
/// (Reverse, только слова — всегда текст, монеты там нет). Кандзи
/// показывают только знак — значение является ответом; грамматика —
/// японскую строку примера без перевода (смысл тоже ответ, спека
/// §Тренировка).
#[component]
pub(super) fn TrainingFrontSlide(
    ctx: AcquaintanceContext,
    card_id: Ulid,
    reverse: bool,
    #[prop(default = false)] audio_front: bool,
) -> impl IntoView {
    let known_kanji = ctx.known_kanji;
    let i18n = use_i18n();
    // Отступы фронта зависят от фазы: пока юзер думает — воздух вокруг
    // вопроса; после раскрытия ответа вопрос сжимается в шапку ответа
    // (баг-репорт: огромные отступы съедали место на стороне ответа).
    let front_class = move || {
        if ctx.showing_answer.get() {
            "text-center py-1"
        } else {
            "text-center pt-8 pb-12 sm:pt-10 sm:pb-16"
        }
    };
    view! {
        <div class=front_class data-testid="acquaintance-training-front">
            {move || {
                let Some(slide) = ctx
                    .slides
                    .get()
                    .iter()
                    .find(|s| s.card_id() == card_id)
                    .cloned()
                else {
                    return ().into_any();
                };
                match slide {
                    AcquaintanceSlideData::Vocabulary { word, translations, .. } => {
                        if audio_front {
                            // Аудио-фронт достижим только на Forward
                            // (инвариант resolve_audio_front): звучит
                            // слово, текст спрятан — вспомнить перевод.
                            // Повтор — явное действие: безтекстовый фронт
                            // без звука нерешаем, мьют его не гейтит. На
                            // стороне ответа кнопка скрыта: слово уже на
                            // экране, повтор — кнопкой в шапке.
                            let ctx_for_replay = ctx.clone();
                            view! {
                                <div class="flex flex-col items-center gap-4">
                                    <Show when=move || !ctx.showing_answer.get()>
                                        <button
                                            data-testid="acquaintance-audio-front-play"
                                            class="audio-player-btn p-3 sm:p-4 rounded-full border transition-all cursor-pointer hover:bg-[var(--bg-hover)]"
                                            on:click=move |_| {
                                                let word = ctx_for_replay
                                                    .slides
                                                    .get_untracked()
                                                    .iter()
                                                    .find(|s| s.card_id() == card_id)
                                                    .and_then(|slide| {
                                                        slide.word().map(str::to_string)
                                                    });
                                                if let Some(word) = word.as_deref() {
                                                    speak_word(word, 1.0);
                                                }
                                            }
                                        >
                                            <Icon icon=icondata::LuVolume2 width="1.5em" height="1.5em" />
                                        </button>
                                    </Show>
                                    <p class="font-mono text-lg text-[var(--fg-muted)]">
                                        {t!(i18n, lesson.listen_word)}
                                    </p>
                                </div>
                            }
                                .into_any()
                        } else if reverse {
                            view! {
                                <p class="font-mono text-3xl text-[var(--fg-black)]">
                                    {translations.join(", ")}
                                </p>
                            }
                                .into_any()
                        } else {
                            view! {
                                <WordTrainingFront
                                    word=word
                                    known_kanji=known_kanji.get_untracked()
                                    native_language=ctx.native_language.get_untracked()
                                />
                            }
                                .into_any()
                        }
                    },
                    // Только знак: значение и чтения — ответ.
                    AcquaintanceSlideData::Kanji { kanji, .. } => view! {
                        <p class="font-serif text-6xl text-[var(--fg-black)]">{kanji}</p>
                    }
                        .into_any(),
                    AcquaintanceSlideData::Grammar { title, examples, .. } => {
                        // Пустые examples — фронт вырождается в заголовок
                        // конструкции («знак» правила, не смысл).
                        // TranslatorText — тот же компонент, что во фразах:
                        // фуригана на токенах и словарные попапы; маркеры
                        // emphasis из examples зачищаются (плоский текст
                        // фронта их не рендерит).
                        let front = grammar_example_front(&examples)
                            .map(|line| strip_emphasis_markers(&line))
                            .unwrap_or_else(|| title.clone());
                        let native_lang: Signal<NativeLanguage> = ctx.native_language.into();
                        let front_class = "font-serif text-3xl text-[var(--fg-black)] leading-relaxed";
                        view! {
                            <TranslatorText
                                text=front
                                class=Signal::derive(move || front_class.to_string())
                                native_language=native_lang
                            />
                        }
                            .into_any()
                    },
                }
            }}
        </div>
    }
}
