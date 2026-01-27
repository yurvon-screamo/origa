use crate::components::cards::grammar_card::{GrammarCard, GrammarCardData, GrammarExample};
use crate::components::cards::vocab_card::CardStatus;
use crate::components::forms::jlpt_level_filter::JlptLevelFilter;
use crate::components::forms::search_bar::SearchBar;
use crate::components::layout::app_layout::{AppLayout, PageHeader};
use leptos::prelude::*;
use origa::domain::JapaneseLevel;

#[component]
pub fn Grammar() -> impl IntoView {
    // Search and filter state
    let (search_query, set_search_query) = signal("".to_string());
    let (selected_level, set_selected_level) = signal(JapaneseLevel::N5);

    // Mock data - will be replaced with real data from use cases
    let mock_grammar = create_mocks();

    // Filter grammar based on search and JLPT level
    let filtered_grammar = Signal::derive(move || {
        let level = selected_level.get();
        let search = search_query.get().to_lowercase();

        mock_grammar
            .iter()
            .filter(|grammar| {
                // Apply JLPT level filter
                let level_match = grammar.jlpt_level == level;

                // Apply search filter
                let search_match = search.is_empty()
                    || grammar.pattern.to_lowercase().contains(&search)
                    || grammar.meaning.to_lowercase().contains(&search)
                    || grammar.examples.iter().any(|e| {
                        e.grammar.to_lowercase().contains(&search)
                            || e.sentence.to_lowercase().contains(&search)
                            || e.translation.to_lowercase().contains(&search)
                    });

                level_match && search_match
            })
            .cloned()
            .collect::<Vec<_>>()
    });

    let handle_search = Callback::new(move |query: String| {
        set_search_query.set(query);
    });

    let handle_level_select = Callback::new(move |level: JapaneseLevel| {
        set_selected_level.set(level);
    });

    let handle_add_grammar = Callback::new(|grammar_id: String| {
        // TODO: Add grammar rule to user's knowledge set
        println!("Add grammar: {}", grammar_id);
    });

    let handle_remove_grammar = Callback::new(|grammar_id: String| {
        // TODO: Remove grammar rule from user's knowledge set
        println!("Remove grammar: {}", grammar_id);
    });

    let handle_grammar_tap = Callback::new(|grammar_id: String| {
        // TODO: Navigate to grammar rule details
        println!("Tap grammar: {}", grammar_id);
    });

    view! {
        <AppLayout active_tab="grammar".to_string()>
            <PageHeader
                title="Грамматика".to_string()
                subtitle="Изучите японские грамматические конструкции"
                    .to_string()
            />

            // Search Bar
            <SearchBar
                placeholder="Поиск грамматической конструкции"
                value=search_query
                on_change=handle_search
            />

            // JLPT Level Filter
            <div class="section">
                <JlptLevelFilter
                    selected_level=selected_level
                    on_select=handle_level_select
                    show_counts=true
                />
            </div>

            // Grammar List
            <div class="section">
                <div class="section-header">
                    <div>
                        <h2 class="section-title">Список конструкций</h2>
                        <p class="section-subtitle">
                            {move || {
                                let level = selected_level.get();
                                let count = filtered_grammar.get().len();
                                format!("{} конструкций уровня {}", count, level)
                            }}
                        </p>
                    </div>
                </div>

                <div class="grammar-grid">
                    <For
                        each=move || filtered_grammar.get()
                        key=|grammar| grammar.id.clone()
                        children=move |grammar| {
                            view! {
                                <GrammarCard
                                    card=grammar
                                    on_add=handle_add_grammar
                                    on_remove=handle_remove_grammar
                                    on_tap=handle_grammar_tap
                                />
                            }
                        }
                    />
                </div>

                // Empty state
                <Show
                    when=move || filtered_grammar.get().is_empty()
                    fallback=|| view! { <div></div> }
                >
                    <div class="empty-state">
                        <div class="empty-icon">{"📝"}</div>
                        <h3 class="empty-title">
                            Грамматические конструкции не найдены
                        </h3>
                        <p class="empty-description">
                            {move || {
                                if search_query.get().is_empty() {
                                    format!(
                                        "В уровне {} пока нет конструкций",
                                        selected_level.get(),
                                    )
                                } else {
                                    format!(
                                        "По запросу \"{}\" ничего не найдено",
                                        search_query.get(),
                                    )
                                }
                            }}
                        </p>
                    </div>
                </Show>
            </div>
        </AppLayout>
    }
}

fn create_mocks() -> Vec<GrammarCardData> {
    vec![
        GrammarCardData {
            id: "grammar_1".to_string(),
            pattern: "～てあげる".to_string(),
            meaning: "Действовать от имени кого-либо / делать что-то для кого-либо".to_string(),
            attachment_rules: "Глагол в форме て + 下さる".to_string(),
            difficulty: 25,
            difficulty_text: "Легко".to_string(),
            jlpt_level: JapaneseLevel::N5,
            examples: vec![
                GrammarExample {
                    grammar: "～てあげる".to_string(),
                    sentence: "先生に本を貸してあげる。".to_string(),
                    translation: "Даю книгу учителю".to_string(),
                    romaji: "Sensei ni hon o kashite ageru.".to_string(),
                },
                GrammarExample {
                    grammar: "～てあげる".to_string(),
                    sentence: "友達に本を貸してあげる。".to_string(),
                    translation: "Даю книгу друзьям".to_string(),
                    romaji: "Tomodachi ni hon o kashite ageru.".to_string(),
                },
            ],
            status: CardStatus::New,
            next_review: chrono::Local::now().date_naive(),
            is_in_knowledge_set: false,
        },
        GrammarCardData {
            id: "grammar_2".to_string(),
            pattern: "～から".to_string(),
            meaning: "От / из (указание на источник или начало действия)".to_string(),
            attachment_rules: "Существительное + から".to_string(),
            difficulty: 35,
            difficulty_text: "Средне".to_string(),
            jlpt_level: JapaneseLevel::N5,
            examples: vec![
                GrammarExample {
                    grammar: "～から".to_string(),
                    sentence: "10時から勉強します。".to_string(),
                    translation: "Буду учиться с 10 часов".to_string(),
                    romaji: "Juuji kara benkyou shimasu.".to_string(),
                },
                GrammarExample {
                    grammar: "～から".to_string(),
                    sentence: "会社から帰ります。".to_string(),
                    translation: "Возвращаюсь с работы".to_string(),
                    romaji: "Kaisha kara kaerimasu.".to_string(),
                },
            ],
            status: CardStatus::InProgress,
            next_review: chrono::Local::now().date_naive() + chrono::Duration::days(2),
            is_in_knowledge_set: true,
        },
        GrammarCardData {
            id: "grammar_3".to_string(),
            pattern: "～なければならない".to_string(),
            meaning: "Если не А, то не Б (необходимость)".to_string(),
            attachment_rules: "Глагол в отрицательной форме (未然形) + なければ + ならない"
                .to_string(),
            difficulty: 60,
            difficulty_text: "Сложно".to_string(),
            jlpt_level: JapaneseLevel::N4,
            examples: vec![GrammarExample {
                grammar: "～なければならない".to_string(),
                sentence: "お金がなければ買えません。".to_string(),
                translation: "Если нет денег, не могу купить".to_string(),
                romaji: "Okane ga nakereba kaemasen.".to_string(),
            }],
            status: CardStatus::Difficult,
            next_review: chrono::Local::now().date_naive() + chrono::Duration::days(1),
            is_in_knowledge_set: true,
        },
        GrammarCardData {
            id: "grammar_4".to_string(),
            pattern: "～はずにはいられない".to_string(),
            meaning: "Невозможно не сделать что-то (негативная логика)".to_string(),
            attachment_rules: "Глагол в форме はず + には + いられない".to_string(),
            difficulty: 75,
            difficulty_text: "Очень сложно".to_string(),
            jlpt_level: JapaneseLevel::N3,
            examples: vec![GrammarExample {
                grammar: "～はずにはいられない".to_string(),
                sentence: "これは信じがたいはずにはいられないことだ。".to_string(),
                translation: "Это то, что невозможно не поверить".to_string(),
                romaji: "Kore wa shinjigatai hazu ni wa irarenai koto da.".to_string(),
            }],
            status: CardStatus::Mastered,
            next_review: chrono::Local::now().date_naive() + chrono::Duration::days(7),
            is_in_knowledge_set: true,
        },
        GrammarCardData {
            id: "grammar_5".to_string(),
            pattern: "～ざるを得ない".to_string(),
            meaning: "Не может сделать что-то, даже если захочет (невозможность)".to_string(),
            attachment_rules: "Глагол в.Dictionary-форме + ざるを得ない".to_string(),
            difficulty: 85,
            difficulty_text: "Экспертно".to_string(),
            jlpt_level: JapaneseLevel::N2,
            examples: vec![GrammarExample {
                grammar: "～ざるを得ない".to_string(),
                sentence: "今から出ても間に合わざるを得ない。".to_string(),
                translation: "Если выйду сейчас, не успею".to_string(),
                romaji: "Ima kara detemo ma ni awazaru o enai.".to_string(),
            }],
            status: CardStatus::New,
            next_review: chrono::Local::now().date_naive(),
            is_in_knowledge_set: false,
        },
    ]
}
