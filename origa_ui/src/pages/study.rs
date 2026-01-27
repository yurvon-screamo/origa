use crate::components::interactive::flash_card::FlashCard;
use crate::components::interactive::flash_card::{
    GrammarCard, KanjiCard, StudyCard, StudyCardWrapper, VocabCard,
};
use crate::components::interactive::navigation::{StudyNavigation, StudySettings};
use crate::components::interactive::next_button::NextButton;
use crate::components::interactive::progress_bar::{CircularProgress, CircularSize, StepIndicator};
use crate::components::interactive::rating_buttons::RatingButtons;
use codee::string::JsonSerdeCodec;
use leptos::prelude::*;
use leptos_use::storage::use_local_storage;
use origa::domain::Rating;

#[component]
pub fn StudySession() -> impl IntoView {
    // Study session state
    let (current_card_index, set_current_card_index) = signal(0);
    let (show_answer, set_show_answer) = signal(false);
    let (selected_rating, set_selected_rating) = signal(Rating::Good);
    let (is_completed, set_is_completed) = signal(false);
    let (show_rating_result, set_show_rating_result) = signal(false);

    // Settings state - persisted with use_local_storage from leptos-use
    let (audio_enabled, set_audio_enabled, _) =
        use_local_storage::<bool, JsonSerdeCodec>("origa_audio_enabled");
    let (auto_advance, set_auto_advance, _) =
        use_local_storage::<bool, JsonSerdeCodec>("origa_auto_advance");
    let (show_answers, set_show_answers, _) =
        use_local_storage::<bool, JsonSerdeCodec>("origa_show_answers");
    let (show_settings, set_show_settings) = signal(false);

    // Mock data - will be replaced with real data from use cases
    let study_cards = create_study_mocks();
    let total_cards = study_cards.len();
    let current_card = Signal::derive(move || study_cards.get(current_card_index.get()).cloned());

    // Actions
    let handle_back = move |_| {
        if let Some(window) = web_sys::window() {
            let _ = window.location().set_href("/");
        }
    };

    let handle_flip = Callback::new(move |_| {
        set_show_answer.update(|shown| *shown = !*shown);
    });

    let handle_rate = Callback::new(move |rating: Rating| {
        set_selected_rating.set(rating);
        set_show_answer.set(true);
        set_show_rating_result.set(false);
    });

    let handle_next = Callback::new(move |_| {
        if current_card_index.get() < total_cards - 1 {
            set_current_card_index.set(current_card_index.get() + 1);
            set_show_answer.set(false);
            set_show_rating_result.set(false);
        } else {
            // Study session completed
            set_is_completed.set(true);
        }
    });

    let handle_complete_session = Callback::new(move |_| {
        // Navigate to completion screen or dashboard
        if let Some(window) = web_sys::window() {
            let _ = window.location().set_href("/dashboard");
        }
    });

    let handle_show_settings = Callback::new(move |_| {
        set_show_settings.update(|s| *s = !*s);
    });

    // Check if this is fixation session
    let is_fixation = web_sys::window()
        .and_then(|w| w.location().href().ok())
        .map(|href| href.contains("fixation"))
        .unwrap_or(false);
    let session_type = if is_fixation {
        "Закрепление"
    } else {
        "Урок"
    };

    view! {
        <div class="study-container">
            <div class="study-header">
                <button class="back-button" on:click=handle_back>
                    <span class="back-icon">{"←"}</span>
                    <span class="back-text">Закрыть</span>
                </button>

                <div class="header-info">
                    <h1 class="session-title">{session_type}</h1>
                    <StepIndicator
                        current=Signal::derive(move || Some(current_card_index.get()))
                        total=total_cards as u32
                        active=Signal::derive(move || !is_completed.get())
                    />
                </div>

                <div class="progress-section">
                    <CircularProgress
                        size=CircularSize::Small
                        percentage=Signal::derive(move || {
                            if total_cards == 0 {
                                0.0
                            } else {
                                ((current_card_index.get() + 1) as f32 / total_cards as f32) * 100.0
                            }
                        })
                    />
                </div>
            </div>

            <main class="study-content">
                <div class="flash-card-section">
                    <FlashCard
                        card=current_card
                        show_answer=Signal::derive(move || show_answer.get())
                        on_flip=handle_flip
                    />
                </div>

                <div class="action-section">
                    <Show when=move || !show_answer.get()>
                        <RatingButtons
                            on_rate=handle_rate
                            show_result=show_rating_result.get()
                            selected_rating=selected_rating.get()
                        />
                    </Show>

                    <Show when=move || show_answer.get()>
                        <div class="next-button-section">
                            <NextButton on_click=handle_next />
                        </div>
                    </Show>

                    // Show rating result animation
                    <Show when=move || show_rating_result.get()>
                        <div class="rating-result">
                            <span class="result-icon">
                                {move || match selected_rating.get() {
                                    Rating::Again => "😵",
                                    Rating::Hard => "😰",
                                    Rating::Good => "😊",
                                    Rating::Easy => "🎉",
                                }}
                            </span>
                            <span class="result-text">
                                {move || match selected_rating.get() {
                                    Rating::Again => "Попробуйте снова",
                                    Rating::Hard => "Нужно больше практики",
                                    Rating::Good => "Хорошая работа!",
                                    Rating::Easy => "Отлично! Супер!",
                                }}
                            </span>
                        </div>
                    </Show>
                </div>
            </main>

            // Study Navigation
            <StudyNavigation
                show_next=!is_completed.get() && !show_answer.get()
                    && current_card_index.get() < total_cards - 1
                show_skip=false
                next_disabled=show_answer.get() || is_completed.get()
                audio_enabled=audio_enabled.get()
                on_next=handle_next
                on_skip=handle_next
                on_audio_toggle=Callback::new(move |_| {
                    set_audio_enabled.update(|audio| *audio = !*audio);
                })
            />

            // Study Settings
            <button
                class="settings-toggle"
                on:click=move |_| handle_show_settings.run(())
                aria-label="Настройки"
            >
                <span class="settings-icon">{"⚙"}</span>
            </button>

            // Settings Panel
            <Show when=move || show_settings.get()>
                <StudySettings
                    audio_enabled=audio_enabled.get()
                    auto_advance=auto_advance.get()
                    show_answers=show_answers.get()
                    on_audio_toggle=Callback::new(move |_| {
                        set_audio_enabled.update(|audio| *audio = !*audio);
                    })
                    on_auto_advance_toggle=Callback::new(move |_| {
                        set_auto_advance.update(|auto| *auto = !*auto);
                    })
                    on_show_answers_toggle=Callback::new(move |_| {
                        set_show_answers.update(|show| *show = !*show);
                    })
                />
            </Show>

            // Empty state
            <Show when=move || current_card.get().is_none()>
                <div class="empty-session">
                    <div class="empty-icon">{"📚"}</div>
                    <h3 class="empty-title">Нет карточек для изучения</h3>
                    <p class="empty-description">
                        Добавьте новые слова, кандзи или грамматические конструкции чтобы начать обучение
                    </p>
                    <button
                        class="settings-button"
                        on:click=move |_| {
                            if let Some(window) = web_sys::window() {
                                let _ = window.location().set_href("/");
                            }
                        }
                    >
                        <span class="back-text">Перейти к библиотеке</span>
                    </button>
                </div>
            </Show>

            // Completion state
            <Show when=move || is_completed.get()>
                <div class="completion-section">
                    <div class="completion-content">
                        <div class="completion-icon">{"🎉"}</div>
                        <h2 class="completion-title">Сессия завершена!</h2>
                        <p class="completion-subtitle">
                            Отличная работа! Вы изучили {total_cards}
                            карточек
                        </p>
                        <div class="completion-stats">
                            <div class="stat-item">
                                <span class="stat-label">Изучено:</span>
                                <span class="stat-value">{total_cards}</span>
                            </div>
                            <div class="stat-item">
                                <span class="stat-label">Время:</span>
                                <span class="stat-value">~{total_cards * 2}мин</span>
                            </div>
                        </div>
                    </div>

                    <div class="completion-actions">
                        <button
                            class="completion-button button-primary"
                            on:click=move |_| handle_complete_session.run(())
                        >
                            Завершить
                        </button>
                        <button
                            class="completion-button secondary"
                            on:click=move |_| {
                                if let Some(window) = web_sys::window() {
                                    let _ = window.location().set_href("/");
                                }
                            }
                        >
                            Продолжить
                        </button>
                    </div>
                </div>
            </Show>
        </div>
    }
}

fn create_study_mocks() -> Vec<StudyCardWrapper> {
    vec![
        StudyCardWrapper {
            card: StudyCard::Vocab(VocabCard {
                japanese: "本".to_string(),
                reading: "ほん".to_string(),
                translation: "книга".to_string(),
                examples: vec![
                    crate::components::interactive::flash_card::VocabExample {
                        japanese: "本を読みます".to_string(),
                        reading: "ほんをよみます".to_string(),
                        translation: "Я читаю книгу".to_string(),
                    },
                    crate::components::interactive::flash_card::VocabExample {
                        japanese: "本を買います".to_string(),
                        reading: "ほんをかいます".to_string(),
                        translation: "Я покупаю книгу".to_string(),
                    },
                ],
            }),
        },
        StudyCardWrapper {
            card: StudyCard::Kanji(KanjiCard {
                character: "日".to_string(),
                stroke_count: 4,
                meanings: vec!["день".to_string(), "солнце".to_string()],
                onyomi: vec!["ニチ".to_string()],
                kunyomi: vec!["ひ".to_string()],
                radicals: vec![],
            }),
        },
        StudyCardWrapper {
            card: StudyCard::Grammar(GrammarCard {
                pattern: "～てあげる".to_string(),
                meaning: "Действовать от имени кого-либо".to_string(),
                attachment_rules: "Глагол в форме て + 下さる".to_string(),
                examples: vec![crate::components::interactive::flash_card::GrammarExample {
                    grammar: "～てあげる".to_string(),
                    sentence: "先生に本を貸してあげる。".to_string(),
                    translation: "Даю книгу учителю".to_string(),
                }],
            }),
        },
    ]
}
