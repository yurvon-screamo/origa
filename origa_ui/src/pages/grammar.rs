use crate::components::cards::grammar_card::GrammarCard;
use crate::components::cards::vocab_card::CardStatus;
use crate::components::forms::jlpt_level_filter::JlptLevelFilter;
use crate::components::forms::search_bar::{FilterChip, FilterChips, SearchBar};
use crate::components::layout::app_layout::{AppLayout, PageHeader};
use crate::services::grammar_service::GrammarService;
use leptos::prelude::*;
use leptos::task::spawn_local;
use origa::domain::JapaneseLevel;

#[component]
pub fn Grammar() -> impl IntoView {
    // Get service from context
    let grammar_service = use_context::<GrammarService>().expect("GrammarService not provided");

    // Search and filter state
    let (search_query, set_search_query) = signal("".to_string());
    let (selected_filter, set_selected_filter) = signal("all".to_string());
    let (selected_level, set_selected_level) = signal(JapaneseLevel::N5);

    // Load grammar data
    let grammar_resource = LocalResource::new({
        let service = grammar_service.clone();
        move || {
            let service = service.clone();
            let level = selected_level.get();
            async move {
                service
                    .get_grammar_by_level(level)
                    .await
                    .unwrap_or_default()
            }
        }
    });

    let grammar_list = Signal::derive(move || grammar_resource.get().unwrap_or_default());

    // Динамические filter chips
    let filter_chips = Signal::derive(move || {
        let cards = grammar_list.get();
        let total = cards.len();
        let new = cards.iter().filter(|c| c.status == CardStatus::New).count();
        let difficult = cards
            .iter()
            .filter(|c| c.status == CardStatus::Difficult)
            .count();
        let in_progress = cards
            .iter()
            .filter(|c| c.status == CardStatus::InProgress)
            .count();
        let mastered = cards
            .iter()
            .filter(|c| c.status == CardStatus::Mastered)
            .count();

        vec![
            FilterChip::new("all", "Все", "📚").with_count(total as u32),
            FilterChip::new("new", "Новые", "🆕").with_count(new as u32),
            FilterChip::new("difficult", "Сложные", "😰").with_count(difficult as u32),
            FilterChip::new("in_progress", "В процессе", "📖").with_count(in_progress as u32),
            FilterChip::new("mastered", "Изученные", "✅").with_count(mastered as u32),
        ]
    });

    // Filter grammar based on search and filter
    let filtered_grammar = Signal::derive(move || {
        let filter = selected_filter.get();
        let search = search_query.get().to_lowercase();

        grammar_list
            .get()
            .iter()
            .filter(|grammar| {
                // Apply status filter
                let status_match = match filter.as_str() {
                    "all" => true,
                    "new" => grammar.status == CardStatus::New,
                    "difficult" => grammar.status == CardStatus::Difficult,
                    "in_progress" => grammar.status == CardStatus::InProgress,
                    "mastered" => grammar.status == CardStatus::Mastered,
                    _ => true,
                };

                // Apply search filter
                let search_match = search.is_empty()
                    || grammar.pattern.to_lowercase().contains(&search)
                    || grammar.meaning.to_lowercase().contains(&search);

                status_match && search_match
            })
            .cloned()
            .collect::<Vec<_>>()
    });

    let handle_search = Callback::new(move |query: String| {
        set_search_query.set(query);
    });

    let handle_filter = Callback::new(move |filter: String| {
        set_selected_filter.set(filter);
    });

    let handle_level_select = Callback::new(move |level: JapaneseLevel| {
        set_selected_level.set(level);
    });

    let handle_add_grammar = {
        let grammar_service = grammar_service.clone();
        Callback::new(move |grammar_id: String| {
            let service = grammar_service.clone();
            if let Ok(rule_id) = grammar_id.parse::<ulid::Ulid>() {
                spawn_local(async move {
                    let _ = service.add_grammar_to_knowledge_set(rule_id).await;
                    // TODO: Обновить список грамматики после добавления
                });
            }
        })
    };

    let handle_remove_grammar = {
        let grammar_service = grammar_service.clone();
        Callback::new(move |grammar_id: String| {
            let service = grammar_service.clone();
            if let Ok(rule_id) = grammar_id.parse::<ulid::Ulid>() {
                spawn_local(async move {
                    let _ = service.remove_grammar_from_knowledge_set(rule_id).await;
                    // TODO: Обновить список грамматики после удаления
                });
            }
        })
    };

    let handle_grammar_tap = Callback::new(|grammar_id: String| {
        // TODO: Navigate to grammar rule details
        println!("Tap grammar: {}", grammar_id);
    });

    view! {
        <AppLayout active_tab="grammar".to_string()>
            <PageHeader
                title=Signal::derive(|| "Грамматика".to_string())
                subtitle="Изучите японские грамматические конструкции"
                    .to_string()
            />

            // Search Bar
            <SearchBar
                placeholder="Поиск грамматической конструкции"
                value=search_query
                on_change=handle_search
            />

            // Category Filter Chips
            <div class="section">
                <FilterChips chips=filter_chips selected=selected_filter on_select=handle_filter />
            </div>

            // JLPT Level Filter
            <div class="section">
                <JlptLevelFilter
                    selected_level=selected_level
                    on_select=handle_level_select
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
