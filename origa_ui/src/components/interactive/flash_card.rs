use leptos::prelude::*;
use leptos::html::on::click;
use crate::components::cards::vocab_card::CardStatus;
use crate::components::cards::kanji_detail::{KanjiDetailData};
use crate::components::cards::grammar_card::{GrammarCardData};

#[component]
pub fn FlashCard(
    card: Signal<Option<StudyCardWrapper>>,
    show_answer: Signal<bool>,
    #[prop(into, optional)] on_flip: Option<Callback<()>>,
    #[prop(into, optional)] disabled: Option<bool>,
) -> impl IntoView {
    let is_disabled = disabled.unwrap_or(false);
    let handle_flip = move |_| {
        if !is_disabled && on_flip.is_some() {
            on_flip.unwrap().run(());
        }
    };
    
    view! {
        <div class="flash-card-container">
            <Show when=move || card().is_some()>
                fallback=|| view! { <div></div> }
            >
                <div 
                    class=format!(
                        "flash-card {} {} {}",
                        if is_disabled { "flash-card-disabled" } else { "" },
                        if show_answer() { "flash-card-flipped" } else { "" }
                    )
                    on:click=handle_flip
                >
                    <div class="flash-card-face flash-card-front">
                        {move || {
                            card().as_ref().map(|wrapper| {
                                match &wrapper.card {
                                    StudyCard::Vocab(vocab) => view! {
                                        <VocabCardContent vocab=vocab />
                                    },
                                    StudyCard::Kanji(kanji) => view! {
                                        <KanjiCardContent kanji=kanji />
                                    },
                                    StudyCard::Grammar(grammar) => view! {
                                        <GrammarCardContent grammar=grammar />
                                    },
                                }
                            }).unwrap_or_else(|| view! { <div></div> })
                        }}
                    </div>
                    
                    <div class="flash-card-face flash-card-back">
                        {move || {
                            card().as_ref().map(|wrapper| {
                                match &wrapper.card {
                                    StudyCard::Vocab(vocab) => view! {
                                        <VocabAnswerContent vocab=vocab />
                                    },
                                    StudyCard::Kanji(kanji) => view! {
                                        <KanjiAnswerContent kanji=kanji />
                                    },
                                    StudyCard::Grammar(grammar) => view! {
                                        <GrammarAnswerContent grammar=grammar />
                                    },
                                }
                            }).unwrap_or_else(|| view! { <div></div> })
                        }}
                    </div>
                </div>
            </Show>
            
            // Empty state when no card
            <Show when=move || card().is_none()>
                fallback=|| view! { <div></div> }
            >
                <div class="flash-card-empty">
                    <div class="empty-content">
                        <div class="empty-icon">📚</div>
                        <h3 class="empty-title">Нет карточек для изучения</h3>
                        <p class="empty-description">
                            Начните с добавления новых слов, кандзи или грамматических конструкций
                        </p>
                    </div>
                </div>
            </Show>
        </div>
    }
}

// Content components for different card types
#[component]
fn VocabCardContent(vocab: &VocabCard) -> impl IntoView {
    let (is_playing, set_is_playing) = create_signal(false);
    
    let handle_audio = move |_| {
        set_is_playing.set(true);
        // In a real app, this would play actual audio
        web_sys::console::log_1(&format!("Playing audio for: {}", vocab.japanese));
        
        // Simulate audio completion
        let vocab_clone = vocab.clone();
        spawn_local(async move {
            gloo_timers::future::sleep(std::time::Duration::from_millis(2000)).await;
            set_is_playing.set(false);
            web_sys::console::log_1(&format!("Audio completed for: {}", vocab_clone.japanese));
        });
    };
    
    view! {
        <div class="vocab-flash-front">
            <div class="flash-character">{vocab.japanese}</div>
            <div class="flash-reading">{vocab.reading}</div>
            <button 
                class="audio-button"
                on:click=handle_audio
                aria-label="Прослушать произношение"
                disabled=is_playing()
            >
                <span class="audio-icon">
                    {move || if is_playing() { "⏸" } else { "🔊" }}
                </span>
                <span class="audio-text">
                    {move || if is_playing() { "Воспроизведение..." } else { "Прослушать" }}
                </span>
            </button>
        </div>
    }
}

#[component]
fn VocabAnswerContent(vocab: &VocabCard) -> impl IntoView {
    view! {
        <div class="vocab-flash-back">
            <div class="answer-header">
                <h4 class="answer-title">{vocab.japanese}</h4>
                <span class="answer-reading">{vocab.reading}</span>
            </div>
            
            <div class="answer-translation">
                <h5 class="translation-title">Перевод:</h5>
                <p class="translation-text">{vocab.translation}</p>
            </div>
            
            <Show when=move || !vocab.examples.is_empty()>
                fallback=|| view! { <div></div> }
            >
                <div class="examples-section">
                    <h6 class="examples-title">Примеры:</h6>
                    <div class="examples-list">
                        {vocab.examples.iter().map(|example| view! {
                            <div class="example-item">
                                <div class="example-japanese">
                                    <span class="example-text">{example.japanese}</span>
                                    <span class="example-reading">{example.reading}</span>
                                </div>
                                <div class="example-translation">{example.translation}</div>
                            </div>
                        }).collect_view()}
                    </div>
                </div>
            </Show>
        </div>
    }
}

#[component]
fn KanjiCardContent(kanji: &KanjiCard) -> impl IntoView {
    view! {
        <div class="kanji-flash-front">
            <div class="flash-character">{kanji.character}</div>
            <div class="flash-stroke-count">{kanji.stroke_count} черт</div>
        </div>
    }
}

#[component]
fn KanjiAnswerContent(kanji: &KanjiCard) -> impl IntoView {
    view! {
        <div class="kanji-flash-back">
            <div class="answer-header">
                <h4 class="answer-title">{kanji.character}</h4>
            </div>
            
            <div class="answer-meanings">
                <h5 class="meanings-title">Значения:</h5>
                <div class="meanings-list">
                    {kanji.meanings.iter().map(|meaning| view! {
                        <span class="meaning-item">{meaning}</span>
                    }).collect_view()}
                </div>
            </div>
            
            <div class="answer-readings">
                <div class="readings-section">
                    <h6 class="readings-title">Onyomi:</h6>
                    <div class="readings-list">
                        {kanji.onyomi.iter().map(|reading| view! {
                            <span class="reading-item">{reading}</span>
                        }).collect_view()}
                    </div>
                </div>
                
                <div class="readings-section">
                    <h6 class="readings-title">Kunyomi:</h6>
                    <div class="readings-list">
                        {kanji.kunyomi.iter().map(|reading| view! {
                            <span class="reading-item">{reading}</span>
                        }).collect_view()}
                    </div>
                </div>
            </div>
            
            <div class="answer-radicals">
                <h6 class="radicals-title">Радикалы:</h6>
                <div class="radicals-list">
                    {kanji.radicals.iter().map(|radical| view! {
                        <div class="radical-item">
                            <span class="radical-char">{radical.character}</span>
                            <span class="radical-meaning">{radical.meaning}</span>
                        </div>
                    }).collect_view()}
                </div>
            </div>
        </div>
    }
}

#[component]
fn GrammarCardContent(grammar: &GrammarCard) -> impl IntoView {
    view! {
        <div class="grammar-flash-front">
            <div class="flash-grammar">{grammar.pattern}</div>
        </div>
    }
}

#[component]
fn GrammarAnswerContent(grammar: &GrammarCard) -> impl IntoView {
    view! {
        <div class="grammar-flash-back">
            <div class="answer-header">
                <h4 class="answer-title">{grammar.pattern}</h4>
            </div>
            
            <div class="answer-meaning">
                <h5 class="meaning-title">Объяснение:</h5>
                <p class="meaning-text">{grammar.meaning}</p>
            </div>
            
            <div class="answer-attachment">
                <h6 class="attachment-title">Правило присоединения:</h6>
                <p class="attachment-text">{grammar.attachment_rules}</p>
            </div>
            
            <Show when=move || !grammar.examples.is_empty()>
                fallback=|| view! { <div></div> }
            >
                <div class="examples-section">
                    <h6 class="examples-title">Примеры:</h6>
                    <div class="examples-list">
                        {grammar.examples.iter().map(|example| view! {
                            <div class="example-item">
                                <div class="example-japanese">
                                    <span class="example-grammar">{example.grammar}</span>
                                    <span class="example-sentence">{example.sentence}</span>
                                </div>
                                <div class="example-translation">{example.translation}</div>
                            </div>
                        }).collect_view()}
                    </div>
                </div>
            </Show>
        </div>
    }
}

// Wrapper types for study session
#[derive(Clone)]
pub enum StudyCard {
    Vocab(VocabCard),
    Kanji(KanjiCard),
    Grammar(GrammarCard),
}

#[derive(Clone)]
pub struct StudyCardWrapper {
    pub card: StudyCard,
    pub status: CardStatus,
    pub difficulty: u32,
    pub stability: u32,
}

// Mock data types (will be replaced with real data)
#[derive(Clone)]
pub struct VocabCard {
    pub id: String,
    pub japanese: String,
    pub reading: String,
    pub translation: String,
    pub examples: Vec<VocabExample>,
}

#[derive(Clone)]
pub struct VocabExample {
    pub japanese: String,
    pub reading: String,
    pub translation: String,
}

#[derive(Clone)]
pub struct KanjiCard {
    pub id: String,
    pub character: String,
    pub stroke_count: u8,
    pub meanings: Vec<String>,
    pub onyomi: Vec<String>,
    pub kunyomi: Vec<String>,
    pub radicals: Vec<RadicalInfo>,
}

#[derive(Clone)]
pub struct RadicalInfo {
    pub character: String,
    pub meaning: String,
}

#[derive(Clone)]
pub struct GrammarCard {
    pub id: String,
    pub pattern: String,
    pub meaning: String,
    pub attachment_rules: String,
    pub examples: Vec<GrammarExample>,
}

#[derive(Clone)]
pub struct GrammarExample {
    pub grammar: String,
    pub sentence: String,
    pub translation: String,
}