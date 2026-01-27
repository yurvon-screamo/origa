use crate::components::cards::vocab_card::{CardStatus, VocabCard, VocabularyCardData};
use crate::components::forms::create_vocab_modal::{CreateVocabularyData, CreateVocabularyModal};
use crate::components::forms::search_bar::{FilterChip, FilterChips, SearchBar};
use crate::components::interactive::floating_button::{FabVariant, FloatingActionButton};
use crate::components::layout::app_layout::{AppLayout, PageHeader};
use leptos::prelude::*;

#[component]
pub fn Vocabulary() -> impl IntoView {
    // Search and filter state
    let (search_query, set_search_query) = signal("".to_string());
    let (selected_filter, set_selected_filter) = signal("all".to_string());

    // Modal state
    let (show_create_modal, set_show_create_modal) = signal(false);

    // Mock data - will be replaced with real data from use cases
    let mock_vocabulary = create_mocks();

    // Filter chips data
    let filter_chips = Signal::derive(move || {
        let chips = vec![
            FilterChip::new("all", "Все", "📚").with_count(156),
            FilterChip::new("new", "Новые", "🆕").with_count(33),
            FilterChip::new("difficult", "Сложные", "😰").with_count(12),
            FilterChip::new("in_progress", "В процессе", "📖").with_count(34),
            FilterChip::new("mastered", "Изученные", "✅").with_count(89),
        ];
        chips
    });

    // Filter vocabulary based on search and filter
    let filtered_vocabulary = Signal::derive(move || {
        let filter = selected_filter.get();
        let search = search_query.get().to_lowercase();

        mock_vocabulary
            .iter()
            .filter(|vocab| {
                // Apply status filter
                let status_match = match filter.as_str() {
                    "all" => true,
                    "new" => vocab.status == CardStatus::New,
                    "difficult" => vocab.status == CardStatus::Difficult,
                    "in_progress" => vocab.status == CardStatus::InProgress,
                    "mastered" => vocab.status == CardStatus::Mastered,
                    _ => true,
                };

                // Apply search filter
                let search_match = search.is_empty()
                    || vocab.japanese.to_lowercase().contains(&search)
                    || vocab.reading.to_lowercase().contains(&search)
                    || vocab.translation.to_lowercase().contains(&search);

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

    let handle_add_word = Callback::new(move |_| {
        set_show_create_modal.set(true);
    });

    let handle_close_modal = Callback::new(move |_| {
        set_show_create_modal.set(false);
    });

    let handle_create_word = Callback::new(move |data: CreateVocabularyData| {
        // TODO: Integrate with CreateVocabularyCardUseCase
        println!("Creating word: {} -> {}", data.japanese, data.translation);
        // Here we would call the actual use case
    });

    let handle_edit = Callback::new(|card_id: String| {
        // TODO: Edit word
        println!("Edit word: {}", card_id);
    });

    let handle_delete = Callback::new(|card_id: String| {
        // TODO: Delete word
        println!("Delete word: {}", card_id);
    });

    let handle_card_tap = Callback::new(|card_id: String| {
        // TODO: Navigate to word details
        println!("Tap word: {}", card_id);
    });

    view! {
        <AppLayout active_tab="vocabulary".to_string()>
            <PageHeader
                title="Слова".to_string()
                subtitle="Ваш словарный запас".to_string()
            />

            // Search Bar
            <SearchBar
                placeholder="Поиск по слову или переводу"
                value=search_query
                on_change=handle_search
            />

            // Filter Chips
            <FilterChips chips=filter_chips selected=selected_filter on_select=handle_filter />

            // Vocabulary List
            <div class="section">
                <For
                    each=move || filtered_vocabulary.get()
                    key=|vocab| vocab.id.clone()
                    children=move |vocab| {
                        view! {
                            <VocabCard
                                card=vocab
                                on_edit=handle_edit
                                on_delete=handle_delete
                                on_tap=handle_card_tap
                            />
                        }
                    }
                />

                // Empty state
                <Show
                    when=move || filtered_vocabulary.get().is_empty()
                    fallback=|| view! { <div></div> }
                >
                    <div class="empty-state">
                        <div class="empty-icon">{"📚"}</div>
                        <h3 class="empty-title">Слова не найдены</h3>
                        <p class="empty-description">
                            {move || {
                                if search_query.get().is_empty() && selected_filter.get() != "all" {
                                    "В этой категории пока нет слов"
                                        .to_string()
                                } else if search_query.get().is_empty() {
                                    "Начните добавлять слова в свой словарь"
                                        .to_string()
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

            // Floating Action Button
            <FloatingActionButton
                icon="+"
                label="Добавить слово"
                on_click=handle_add_word
                variant=FabVariant::Primary
            />

            // Create Vocabulary Modal
            <CreateVocabularyModal
                show=Signal::derive(move || show_create_modal.get())
                on_close=handle_close_modal
                on_create=handle_create_word
            />
        </AppLayout>
    }
}

fn create_mocks() -> Vec<VocabularyCardData> {
    vec![
        VocabularyCardData {
            id: "1".to_string(),
            japanese: "本".to_string(),
            reading: "ほん".to_string(),
            translation: "книга".to_string(),
            status: CardStatus::InProgress,
            difficulty: 45,
            stability: 60,
            next_review: chrono::Local::now().date_naive() + chrono::Duration::days(2),
        },
        VocabularyCardData {
            id: "2".to_string(),
            japanese: "食べる".to_string(),
            reading: "たべる".to_string(),
            translation: "есть, кушать".to_string(),
            status: CardStatus::Mastered,
            difficulty: 20,
            stability: 85,
            next_review: chrono::Local::now().date_naive() + chrono::Duration::days(7),
        },
        VocabularyCardData {
            id: "3".to_string(),
            japanese: "勉強".to_string(),
            reading: "べんきょう".to_string(),
            translation: "учиться".to_string(),
            status: CardStatus::Difficult,
            difficulty: 75,
            stability: 30,
            next_review: chrono::Local::now().date_naive() + chrono::Duration::days(1),
        },
        VocabularyCardData {
            id: "4".to_string(),
            japanese: "学校".to_string(),
            reading: "がっこう".to_string(),
            translation: "школа".to_string(),
            status: CardStatus::New,
            difficulty: 50,
            stability: 50,
            next_review: chrono::Local::now().date_naive(),
        },
        VocabularyCardData {
            id: "5".to_string(),
            japanese: "友達".to_string(),
            reading: "ともだち".to_string(),
            translation: "друг".to_string(),
            status: CardStatus::InProgress,
            difficulty: 35,
            stability: 70,
            next_review: chrono::Local::now().date_naive() + chrono::Duration::days(3),
        },
    ]
}
