use leptos::prelude::*;
use leptos_use::use_timeout_fn;

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
            <Show when=move || card.get().is_some()>
                <div
                    class=format!(
                        "flash-card {} {}",
                        if is_disabled { "flash-card-disabled" } else { "" },
                        if show_answer.get() { "flash-card-flipped" } else { "" },
                    )
                    on:click=handle_flip
                >
                    <div class="card-type-indicator">
                        {move || {
                            let card_type = match card.get() {
                                Some(wrapper) => match &wrapper.card {
                                    StudyCard::Vocab(_) => "📚 Слово",
                                    StudyCard::Kanji(_) => "🈁 Кандзи",
                                    StudyCard::Grammar(_) => "📝 Грамматика",
                                },
                                None => "",
                            };
                            card_type
                        }}
                    </div>
                    <div class="flash-card-face flash-card-front">
                        {move || {
                            let result: leptos::prelude::AnyView = card
                                .get()
                                .as_ref()
                                .map(|wrapper| { render_card_content(&wrapper.card) })
                                .unwrap_or_else(|| view! { <div></div> }.into_any());
                            result
                        }}
                    </div>

                    <div class="flash-card-face flash-card-back">
                        {move || {
                            let result: leptos::prelude::AnyView = card
                                .get()
                                .as_ref()
                                .map(|wrapper| { render_answer_content(&wrapper.card) })
                                .unwrap_or_else(|| view! { <div></div> }.into_any());
                            result
                        }}
                    </div>
                </div>
            </Show>

            // Empty state when no card
            <Show when=move || card.get().is_none()>
                <div class="flash-card-empty">
                    <div class="empty-content">
                        <div class="empty-icon">{"📚"}</div>
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

fn render_card_content(card: &StudyCard) -> leptos::prelude::AnyView {
    match card {
        StudyCard::Vocab(vocab) => view! { <VocabCardContent vocab=vocab.clone() /> }.into_any(),
        StudyCard::Kanji(kanji) => view! { <KanjiCardContent kanji=kanji.clone() /> }.into_any(),
        StudyCard::Grammar(grammar) => {
            view! { <GrammarCardContent grammar=grammar.clone() /> }.into_any()
        }
    }
}

fn render_answer_content(card: &StudyCard) -> leptos::prelude::AnyView {
    match card {
        StudyCard::Vocab(vocab) => view! { <VocabAnswerContent vocab=vocab.clone() /> }.into_any(),
        StudyCard::Kanji(kanji) => view! { <KanjiAnswerContent kanji=kanji.clone() /> }.into_any(),
        StudyCard::Grammar(grammar) => {
            view! { <GrammarAnswerContent grammar=grammar.clone() /> }.into_any()
        }
    }
}

// Content components for different card types
#[component]
fn VocabCardContent(vocab: VocabCard) -> impl IntoView {
    let (is_playing, set_is_playing) = signal(false);
    let vocab_reading = vocab.reading.clone();
    let vocab_japanese = vocab.japanese.clone();
    let vocab_japanese_for_log = vocab_japanese.clone();

    // Use leptos-use timeout for audio completion simulation
    let audio_timeout = use_timeout_fn(
        move |_: ()| {
            set_is_playing.set(false);
            web_sys::console::log_1(
                &format!("Audio completed for: {}", vocab_japanese_for_log).into(),
            );
        },
        2000.0,
    );

    let handle_audio = move |_| {
        set_is_playing.set(true);
        // In a real app, this would play actual audio
        web_sys::console::log_1(&format!("Playing audio for: {}", vocab_japanese.clone()).into());
        // Start the timeout to simulate audio completion
        (audio_timeout.start)(());
    };

    view! {
        <div class="vocab-flash-front">
            <div class="flash-character">{vocab.japanese}</div>
            <div class="flash-reading">{vocab_reading}</div>
            <button
                class="audio-button"
                on:click=handle_audio
                aria-label="Прослушать произношение"
                disabled=is_playing.get()
            >
                <span class="audio-icon">
                    {move || if is_playing.get() { "⏸" } else { "🔊" }}
                </span>
                <span class="audio-text">
                    {move || {
                        if is_playing.get() {
                            "Воспроизведение..."
                        } else {
                            "Прослушать"
                        }
                    }}
                </span>
            </button>
        </div>
    }
}

#[component]
fn VocabAnswerContent(vocab: VocabCard) -> impl IntoView {
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

            {move || {
                let examples = vocab.examples.clone();
                let examples_for_check = examples.clone();
                view! {
                    {(!examples_for_check.is_empty())
                        .then(|| {
                            let examples_for_iter = examples;
                            view! {
                                <div class="examples-section">
                                    <h6 class="examples-title">Примеры:</h6>
                                    <div class="examples-list">
                                        {examples_for_iter
                                            .iter()
                                            .map(|example| {
                                                let jp = example.japanese.clone();
                                                let rd = example.reading.clone();
                                                let tr = example.translation.clone();
                                                view! {
                                                    <div class="example-item">
                                                        <div class="example-japanese">
                                                            <span class="example-text">{jp}</span>
                                                            <span class="example-reading">{rd}</span>
                                                        </div>
                                                        <div class="example-translation">{tr}</div>
                                                    </div>
                                                }
                                            })
                                            .collect_view()}
                                    </div>
                                </div>
                            }
                        })}
                }
            }}
        </div>
    }
}

#[component]
fn KanjiCardContent(kanji: KanjiCard) -> impl IntoView {
    view! {
        <div class="kanji-flash-front">
            <div class="flash-character">{kanji.character}</div>
            <div class="flash-stroke-count">{kanji.stroke_count}черт</div>
        </div>
    }
}

#[component]
fn KanjiAnswerContent(kanji: KanjiCard) -> impl IntoView {
    view! {
        <div class="kanji-flash-back">
            <div class="answer-header">
                <h4 class="answer-title">{kanji.character}</h4>
            </div>

            <div class="answer-meanings">
                <h5 class="meanings-title">Значения:</h5>
                <div class="meanings-list">
                    {kanji
                        .meanings
                        .iter()
                        .map(|meaning| {
                            view! { <span class="meaning-item">{meaning.clone()}</span> }
                        })
                        .collect_view()}
                </div>
            </div>

            <div class="answer-readings">
                <div class="readings-section">
                    <h6 class="readings-title">Onyomi:</h6>
                    <div class="readings-list">
                        {kanji
                            .onyomi
                            .iter()
                            .map(|reading| {
                                view! { <span class="reading-item">{reading.clone()}</span> }
                            })
                            .collect_view()}
                    </div>
                </div>

                <div class="readings-section">
                    <h6 class="readings-title">Kunyomi:</h6>
                    <div class="readings-list">
                        {kanji
                            .kunyomi
                            .iter()
                            .map(|reading| {
                                view! { <span class="reading-item">{reading.clone()}</span> }
                            })
                            .collect_view()}
                    </div>
                </div>
            </div>

            <div class="answer-radicals">
                <h6 class="radicals-title">Радикалы:</h6>
                <div class="radicals-list">
                    {kanji
                        .radicals
                        .iter()
                        .map(|radical| {
                            let ch = radical.character.clone();
                            let mn = radical.meaning.clone();
                            view! {
                                <div class="radical-item">
                                    <span class="radical-char">{ch}</span>
                                    <span class="radical-meaning">{mn}</span>
                                </div>
                            }
                        })
                        .collect_view()}
                </div>
            </div>
        </div>
    }
}

#[component]
fn GrammarCardContent(grammar: GrammarCard) -> impl IntoView {
    view! {
        <div class="grammar-flash-front">
            <div class="flash-grammar">{grammar.pattern}</div>
        </div>
    }
}

#[component]
fn GrammarAnswerContent(grammar: GrammarCard) -> impl IntoView {
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

            {move || {
                let examples = grammar.examples.clone();
                let examples_for_check = examples.clone();
                view! {
                    {(!examples_for_check.is_empty())
                        .then(|| {
                            let examples_for_iter = examples;
                            view! {
                                <div class="examples-section">
                                    <h6 class="examples-title">Примеры:</h6>
                                    <div class="examples-list">
                                        {examples_for_iter
                                            .iter()
                                            .map(|example| {
                                                let gr = example.grammar.clone();
                                                let st = example.sentence.clone();
                                                let tr = example.translation.clone();
                                                view! {
                                                    <div class="example-item">
                                                        <div class="example-japanese">
                                                            <span class="example-grammar">{gr}</span>
                                                            <span class="example-sentence">{st}</span>
                                                        </div>
                                                        <div class="example-translation">{tr}</div>
                                                    </div>
                                                }
                                            })
                                            .collect_view()}
                                    </div>
                                </div>
                            }
                        })}
                }
            }}
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
    pub card_id: ulid::Ulid,
    pub card: StudyCard,
}

#[derive(Clone)]
pub struct VocabCard {
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
