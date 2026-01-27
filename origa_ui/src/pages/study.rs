use leptos::prelude::*;
use leptos_router::use_navigate;
use crate::components::layout::app_layout::AppLayout;
use crate::components::interactive::flash_card::FlashCard;
use crate::components::interactive::rating_buttons::RatingButtons;
use crate::components::interactive::progress_bar::{ProgressBar, StepIndicator, CircularSize, CircularProgress};
use crate::components::interactive::flash_card::{StudyCard, StudyCardWrapper, VocabCard, KanjiCard, GrammarCard};
use crate::components::interactive::navigation::{StudyNavigation, SessionProgress, StudySettings};
use origa::domain::Rating;
use crate::components::interactive::progress_bar::CircularProgress;

#[component]
pub fn StudySession() -> impl IntoView {
    // Study session state
    let (current_card_index, set_current_card_index) = create_signal(0);
    let (show_answer, set_show_answer) = create_signal(false);
    let (selected_rating, set_selected_rating) = create_signal(Rating::Good);
    let (is_completed, set_is_completed) = create_signal(false);
    let (show_rating_result, set_show_rating_result) = create_signal(false);
    
    // Settings state
    let (audio_enabled, set_audio_enabled) = create_signal(true);
    let (auto_advance, set_auto_advance) = create_signal(false);
    let (show_answers, set_show_answers) = create_signal(false);
    let (show_settings, set_show_settings) = create_signal(false);
    
    // Mock data - will be replaced with real data from use cases
    let study_cards = create_study_mocks();
    let total_cards = study_cards.len();
    let current_card = Signal::derive(move || {
        study_cards.get(current_card_index()).cloned()
    });
    
    // Actions
    let navigate_back = leptos_router::use_navigate();
    let handle_back = Callback::new(move |_| {
        navigate_back("/", Default::default());
    });
    
    let handle_flip = Callback::new(move |_| {
        set_show_answer.update(|shown| *shown = !*shown);
    });
    
    let handle_rate = create_action(move |rating: Rating| {
        set_selected_rating.set(rating);
        set_show_answer.set(true);
        set_show_rating_result.set(false);
    });
    
    let handle_next = Callback::new(move |_| {
        if current_card_index() < total_cards - 1 {
            set_current_card_index.update(|i| *i + 1);
            set_show_answer.set(false);
            set_show_rating_result.set(false);
        } else {
            // Study session completed
            set_is_completed.set(true);
        }
    });
    
    let handle_complete_session = Callback::new(move |_| {
        // Navigate to completion screen or dashboard
        navigate_back("/dashboard", Default::default());
    });
    
    // Check if this is fixation session
    let is_fixation = leptos_router::use_location().pathname.get().contains("fixation");
    let session_type = if is_fixation { "Закрепление" } else { "Урок" };
    
    view! {
        <div class="study-container">
            // Header with progress
            <div class="study-header">
                <button class="back-button" on:click=handle_back>
                    <span class="back-icon">←</span>
                    <span class="back-text">Закрыть</span>
                </button>
                
                <div class="header-info">
                    <h1 class="session-title">{session_type}</h1>
                    <StepIndicator 
                        current=Signal::derive(move || Some(current_card_index()))
                        total=total_cards 
                        active=Signal::derive(move || !is_completed()) />
                </div>
                
                <div class="progress-section">
                    <CircularProgress 
                        size=CircularSize::Small
                        percentage=Signal::derive(move || {
                            if total_cards == 0 { 0.0 } else { ((current_card_index() + 1) as f32 / total_cards as f32) * 100.0 }
                        }) />
                </div>
            </div>
            
                <main class="study-content">
                        <div class="flash-card-section">
                            <FlashCard 
                                card=current_card
                                show_answer=show_answer
                                on_flip=handle_flip />
                            />
                        </div>
                        
                        <div class="action-section">
                            <Show when=move || !show_answer()>
                                fallback=|| view! { <div></div> }
                            >
                                <RatingButtons 
                                    on_rate=handle_rate
                                    show_result=Signal::derive(move || Some(show_rating_result()))
                                    selected_rating=selected_rating />
                                />
                            </Show>
                            
                            <Show when=move || show_answer()>
                                fallback=|| view! { <div></div> }
                            >
                                <div class="next-button-section">
                                    <NextButton 
                                        on_click=handle_next />
                                    </div>
                            </Show>
                            
                            // Show rating result animation
                            <Show when=show_rating_result()>
                                <div class="rating-result">
                                    <span class="result-icon">
                                        {move || match selected_rating() {
                                            Rating::Again => "😵",
                                            Rating::Hard => "😰",
                                            Rating::Good => "😊",
                                            Rating::Easy => "🎉",
                                        }}
                                    </span>
                                    <span class="result-text">
                                        {move || match selected_rating() {
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
                        current=Signal::derive(move || Some(current_card_index()))
                        total=total_cards
                        show_next=Signal::derive(move || !is_completed() && !show_answer() && current_card_index() < total_cards - 1)
                        show_skip=Signal::derive(move || false)
                        next_disabled=Signal::derive(move || show_answer() || is_completed())
                        audio_enabled=Signal::derive(move || audio_enabled())
                        on_next=handle_next
                        on_skip=Callback::new(|_| {
                            // Skip to next card
                            handle_next();
                        })
                        on_audio_toggle=Callback::new(|_| {
                            set_is_audio_enabled.update(|audio| *audio = !*audio);
                        })
                    />
                </Show>
                
                // Study Settings
                <button 
                    class="settings-toggle"
                    on:click=handle_show_settings
                    aria-label="Настройки"
                >
                    <span class="settings-icon">⚙️</span>
                </button>
                
                // Settings Panel
                <Show when=move || show_settings()>
                    <StudySettings 
                        audio_enabled=audio_enabled
                        auto_advance=Signal::derive(move || false)
                        show_answers=Signal::derive(move || false)
                        on_audio_toggle=Callback::new(move |_| {
                            set_is_audio_enabled.update(|audio| *audio = !*audio);
                        })
                        on_auto_advance_toggle=Callback::new(move |_| {
                            set_auto_advance.update(|auto| *auto = !*auto);
                        })
                        on_show_answers_toggle=Callback::new(move |_| {
                            set_show_answers.update(|show| *show = !*show);
                        })
                        on_save_settings=Callback::new(|_| {
                            // Save settings logic
                            web_sys::console::log_1("Settings saved");
                            set_show_settings.set(false);
                        })
                    />
                </Show>
                
                // Empty state
                <Show when=move || current_card().is_none()>
                    fallback=|| view! { <div></div> }
                >
                    <div class="empty-session">
                        <div class="empty-icon">📚</div>
                        <h3 class="empty-title">Нет карточек для изучения</h3>
                        <p class="empty-description">
                            Добавьте новые слова, кандзи или грамматические конструкции чтобы начать обучение
                        </p>
                    </div>
                </Show>
                
                // Completion state
                <Show when=move || is_completed()>
                    fallback=|| view! { <div></div> }
                >
                    <div class="completion-section">
                        <div class="completion-content">
                            <div class="completion-icon">🎉</div>
                            <h2 class="completion-title">Сессия завершена!</h2>
                            <p class="completion-subtitle">
                                Отличная работа! Вы изучили {total_cards} карточек
                            </p>
                            <div class="completion-stats">
                                <div class="stat-item">
                                    <span class="stat-label">Изучено:</span>
                                    <span class="stat-value">{total_cards}</span>
                                </div>
                                <div class="study-time">
                                    <span class="time-label">Время:</span>
                                    <span class="time-value">~{total_cards * 2} мин</span>
                                </div>
                            </div>
                        </div>
                        
                        <div class="completion-actions">
                            <button class="completion-button" on:click=handle_complete_session>
                                Завершить
                            </button>
                            <button class="completion-button secondary" on:click=handle_back>
                                Продолжить
                            </button>
                        </div>
                    </div>
                </Show>
            </div>
    }
}
                        
                        <Show when=move || show_answer()>
                            fallback=|| view! { <div></div> }
                        >
                            <div class="next-button-section">
                                <button class="next-button" on:click=handle_next>
                                    <span class="next-text">Далее</span>
                                    <span class="next-icon">→</span>
                                </button>
                            </div>
                            
                            // Show rating result animation
                            <Show when=show_rating_result>
                                <div class="rating-result">
                                    <span class="result-icon">
                                        {move || match selected_rating() {
                                            Rating::Again => "😵",
                                            Rating::Hard => "😰",
                                            Rating::Good => "😊",
                                            Rating::Easy => "🎉",
                                        }}
                                    </span>
                                    <span class="result-text">
                                        {move || match selected_rating() {
                                            Rating::Again => "Попробуйте снова",
                                            Rating::Hard => "Нужно больше практики",
                                            Rating::Good => "Хорошая работа!",
                                            Rating::Easy => "Отлично! Супер!",
                                        }}
                                    </span>
                                </div>
                            </Show>
                        </Show>
                    </div>
                </Show>
                
                // Empty state
                <Show when=move || current_card().is_none()>
                    fallback=|| view! { <div></div> }
                >
                    <div class="empty-session">
                        <div class="empty-icon">📚</div>
                        <h3 class="empty-title">Нет карточек для изучения</h3>
                        <p class="empty-description">
                            Добавьте новые слова, кандзи или грамматические конструкции чтобы начать обучение
                        </p>
                        <button class="button button-primary" on:click=handle_back>
                            <span class="back-text">Перейти к библиотеке</span>
                        </button>
                    </div>
                </Show>
            </main>
            
            <Show when=move || is_completed()>
                fallback=|| view! { <div></div> }
                >
                    <div class="completion-section">
                        <div class="completion-content">
                            <div class="completion-icon">🎉</div>
                            <h2 class="completion-title">Сессия завершена!</h2>
                            <p class="completion-subtitle">
                                Отличная работа! Вы изучили {total_cards} карточек
                            </p>
                            <div class="completion-stats">
                                <div class="stat-item">
                                    <span class="stat-label">Изучено:</span>
                                    <span class="stat-value">{total_cards}</span>
                                </div>
                                <div class="stat-item">
                                    <span class="stat-label">Время:</span>
                                    <span class="stat-value">~{total_cards * 2} мин</span>
                                </div>
                            </div>
                        </div>
                        <button class="completion-button button-primary" on:click=handle_complete_session>
                            Завершить
                        </button>
                    </div>
                </Show>
            </div>
        </div>
    }
}

fn create_study_mocks() -> Vec<StudyCardWrapper> {
    vec![
        StudyCardWrapper {
            card: StudyCard::Vocab(VocabCard {
                id: "vocab_1".to_string(),
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
            status: crate::components::cards::vocab_card::CardStatus::InProgress,
            difficulty: 45,
            stability: 60,
        },
    },
    StudyCardWrapper {
            card: StudyCard::Kanji(KanjiCard {
                id: "kanji_1".to_string(),
                character: "日".to_string(),
                stroke_count: 4,
                meanings: vec!["день".to_string(), "солнце".to_string()],
                onyomi: vec!["ニチ".to_string()],
                kunyomi: vec!["ひ".to_string()],
            }),
            status: crate::components::cards::vocab_card::CardStatus::New,
            difficulty: 30,
            stability: 50,
        },
    },
    StudyCardWrapper {
            card: StudyCard::Grammar(GrammarCard {
                id: "grammar_1".to_string(),
                pattern: "～てあげる".to_string(),
                meaning: "Действовать от имени кого-либо".to_string(),
                attachment_rules: "Глагол в форме て + 下さる".to_string(),
                examples: vec![
                    crate::components::interactive::flash_card::GrammarExample {
                        grammar: "～てあげる".to_string(),
                        sentence: "先生に本を貸してあげる。".to_string(),
                        translation: "Даю книгу учителю".to_string(),
                    },
                ],
            }),
            status: crate::components::cards::vocab_card::CardStatus::Difficult,
            difficulty: 75,
            stability: 35,
        },
    ],
]
}