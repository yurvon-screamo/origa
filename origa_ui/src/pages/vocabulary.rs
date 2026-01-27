use crate::components::cards::vocab_card::{CardStatus, VocabCard};
use crate::components::forms::create_vocab_modal::{CreateVocabularyData, CreateVocabularyModal};
use crate::components::forms::search_bar::{FilterChip, FilterChips, SearchBar};
use crate::components::interactive::floating_button::{FabVariant, FloatingActionButton};
use crate::components::layout::app_layout::{AppLayout, PageHeader};
use crate::services::vocabulary_service::VocabularyService;
use leptos::prelude::*;
use leptos::task::spawn_local;

#[component]
pub fn Vocabulary() -> impl IntoView {
    // Search and filter state
    let (search_query, set_search_query) = signal("".to_string());
    let (selected_filter, set_selected_filter) = signal("all".to_string());

    // Modal state
    let (show_create_modal, set_show_create_modal) = signal(false);

    let vocabulary_service =
        use_context::<VocabularyService>().expect("VocabularyService not provided");

    let vocabulary_resource = LocalResource::new({
        let service = vocabulary_service.clone();
        move || {
            let service = service.clone();
            async move {
                let user_id = ulid::Ulid::new(); // TODO: получить реальный user_id
                service
                    .get_user_vocabulary(user_id)
                    .await
                    .unwrap_or_default()
            }
        }
    });

    let vocabulary_list = Signal::derive(move || vocabulary_resource.get().unwrap_or_default());

    // Динамические filter chips
    let filter_chips = Signal::derive({
        let service = vocabulary_service.clone();
        move || {
            let cards = vocabulary_list.get();
            let stats = service.get_vocabulary_stats(&cards);
            vec![
                FilterChip::new("all", "Все", "📚").with_count(stats.total as u32),
                FilterChip::new("new", "Новые", "🆕").with_count(stats.new as u32),
                FilterChip::new("difficult", "Сложные", "😰").with_count(stats.difficult as u32),
                FilterChip::new("in_progress", "В процессе", "📖")
                    .with_count(stats.in_progress as u32),
                FilterChip::new("mastered", "Изученные", "✅").with_count(stats.mastered as u32),
            ]
        }
    });

    // Filter vocabulary based on search and filter
    let filtered_vocabulary = Signal::derive(move || {
        let filter = selected_filter.get();
        let search = search_query.get().to_lowercase();

        vocabulary_list
            .get()
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

    let handle_create_word = {
        let vocabulary_service = vocabulary_service.clone();
        Callback::new(move |data: CreateVocabularyData| {
            let service = vocabulary_service.clone();
            let user_id = ulid::Ulid::new(); // TODO: получить реальный user_id
            spawn_local(async move {
                let _ = service
                    .create_vocabulary(user_id, data.japanese, data.translation)
                    .await;
                // TODO: Обновить список карточек после создания
            });
        })
    };

    let handle_edit = Callback::new(|card_id: String| {
        // TODO: Edit word
        println!("Edit word: {}", card_id);
    });

    let handle_delete = {
        let vocabulary_service = vocabulary_service.clone();
        Callback::new(move |card_id: String| {
            let service = vocabulary_service.clone();
            let user_id = ulid::Ulid::new(); // TODO: получить реальный user_id
            if let Ok(card_id_ulid) = card_id.parse::<ulid::Ulid>() {
                spawn_local(async move {
                    let _ = service.delete_vocabulary(user_id, card_id_ulid).await;
                    // TODO: Обновить список карточек после удаления
                });
            }
        })
    };

    let handle_card_tap = Callback::new(|card_id: String| {
        // TODO: Navigate to word details
        println!("Tap word: {}", card_id);
    });

    view! {
        <AppLayout active_tab="vocabulary".to_string()>
            <PageHeader
                title=Signal::derive(|| "Слова".to_string())
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
