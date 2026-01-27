use leptos::prelude::*;
use crate::components::layout::app_layout::{AppLayout, PageHeader};
use crate::components::forms::search_bar::SearchBar;
use crate::components::forms::jlpt_level_filter::{JlptLevelFilter, JlptLevel};
use crate::components::cards::kanji_card::{KanjiCard, KanjiCardData, RadicalInfo};
use crate::components::cards::vocab_card::CardStatus;
use crate::components::interactive::floating_button::{FloatingActionButton, FabVariant};
use crate::services::kanji_service::{KanjiService, KanjiListData};

#[component]
pub fn Kanji() -> impl IntoView {
    // Get kanji service from context
    let kanji_service = expect_context::<KanjiService>();
    
    // User ID - in a real app, this would come from auth context
    let user_id = Ulid::new();
    
    // Search and filter state
    let (search_query, set_search_query) = create_signal("".to_string());
    let (selected_level, set_selected_level) = create_signal(JlptLevel::N5);
    
    // Loading states
    let (is_loading, set_is_loading) = create_signal(false);
    let (error, set_error) = create_signal(None::<String>);
    
    // Kanji data from service
    let (kanji_data, set_kanji_data) = create_signal(Vec::<KanjiListData>::new());
    
    // Load kanji data when level changes
    let load_kanji = create_action(move |level: JlptLevel| {
        let service = kanji_service.clone();
        let user = user_id;
        async move {
            set_is_loading.set(true);
            set_error.set(None);
            
            match service.get_user_kanji_by_level(user, level).await {
                Ok(kanji) => {
                    set_kanji_data.set(kanji);
                    set_is_loading.set(false);
                }
                Err(e) => {
                    set_error.set(Some(format!("Ошибка загрузки кандзи: {}", e)));
                    set_is_loading.set(false);
                }
            }
        }
    });
    
    // Load initial kanji data
    create_effect(move |_| {
        load_kanji.dispatch(selected_level());
    });
    
    // Filter kanji based on search and JLPT level
    let filtered_kanji = Signal::derive(move || {
        let search = search_query().to_lowercase();
        let data = kanji_data();
        
        data.iter()
            .filter(|kanji| {
                // Apply search filter (level is already filtered by service)
                let search_match = search.is_empty() || 
                    kanji.character.to_lowercase().contains(&search) ||
                    kanji.meanings.iter().any(|m| m.to_lowercase().contains(&search)) ||
                    kanji.onyomi.iter().any(|o| o.to_lowercase().contains(&search)) ||
                    kanji.kunyomi.iter().any(|k| k.to_lowercase().contains(&search)) ||
                    kanji.radicals.iter().any(|r| r.character.to_lowercase().contains(&search) || r.meaning.to_lowercase().contains(&search));
                
                search_match
            })
            .cloned()
            .collect::<Vec<_>>()
    });
    
    let handle_search = Callback::new(move |query: String| {
        set_search_query.set(query);
    });
    
    let handle_level_select = Callback::new(move |level: JlptLevel| {
        set_selected_level.set(level);
    });
    
    let handle_add_kanji = create_action(move |kanji_data: KanjiListData| {
        let service = kanji_service.clone();
        let user = user_id;
        async move {
            match service.add_kanji_to_knowledge_set(user, kanji_data.character.clone()).await {
                Ok(()) => {
                    // Reload kanji data
                    load_kanji.dispatch(selected_level());
                }
                Err(e) => {
                    set_error.set(Some(format!("Не удалось добавить кандзи: {}", e)));
                }
            }
        }
    });
    
    let handle_remove_kanji = create_action(move |kanji_data: KanjiListData| {
        let service = kanji_service.clone();
        let user = user_id;
        async move {
            match service.remove_kanji_from_knowledge_set(user, kanji_data.character.clone()).await {
                Ok(()) => {
                    // Reload kanji data
                    load_kanji.dispatch(selected_level());
                }
                Err(e) => {
                    set_error.set(Some(format!("Не удалось удалить кандзи: {}", e))));
                }
            }
        }
    });
    
    let handle_kanji_tap = Callback::new(move |kanji_id: String| {
        // Navigate to kanji details - this would open a detail page/modal
        // For now, just log it
        web_sys::console::log_1(&format!("Tap kanji: {}", kanji_id));
    });
    
    view! {
        <AppLayout active_tab="kanji".to_string()>
            <PageHeader 
                title="Кандзи" 
                subtitle="Изучите японские иероглифы" />
            
            // Search Bar
            <SearchBar 
                placeholder="Поиск кандзи или значения"
                value=search_query
                on_change=handle_search />
            
            // JLPT Level Filter
            <div class="section">
                <JlptLevelFilter 
                    selected_level=Signal::derive(move || Some(selected_level()))
                    on_select=handle_level_select
                    show_counts=true />
            </div>
            
            // Kanji List
            <div class="section">
                <div class="section-header">
                    <div>
                        <h2 class="section-title">Список кандзи</h2>
                        <p class="section-subtitle">
                            {move || {
                                let level = selected_level();
                                let count = filtered_kanji().len();
                                format!("{} кандзи уровня {}", count, level.to_string())
                            }}
                        </p>
                    </div>
                </div>
                
                <div class="kanji-grid">
                    <For
                        each=filtered_kanji
                        key=|kanji| kanji.id.clone()
                        children=move |kanji| {
                            view! {
                                <KanjiCard 
                                    card=kanji
                                    on_add=handle_add_kanji
                                    on_remove=handle_remove_kanji
                                    on_tap=handle_kanji_tap />
                            }
                        }
                    />
                </div>
                
                // Loading state
                <Show when=move || is_loading()>
                    <div class="loading-state">
                        <div class="spinner"></div>
                        <p class="loading-text">Загрузка кандзи...</p>
                    </div>
                </Show>
                
                // Error state
                <Show when=move || error().is_some()>
                    <div class="error-state">
                        <div class="error-icon">⚠️</div>
                        <h3 class="error-title">Ошибка загрузки</h3>
                        <p class="error-description">
                            {move || error().clone().unwrap_or_default()}
                        </p>
                    </div>
                </Show>
                
                // Empty state
                <Show
                    when=move || !is_loading() && error().is_none() && filtered_kanji().is_empty()
                    fallback=|| view! { <div></div> }
                >
                    <div class="empty-state">
                        <div class="empty-icon">🈁</div>
                        <h3 class="empty-title">Кандзи не найдены</h3>
                        <p class="empty-description">
                            {move || if search_query().is_empty() {
                                format!("В уровне {} пока нет кандзи", selected_level().to_string())
                            } else {
                                format!("По запросу \"{}\" ничего не найдено", search_query())
                            }}
                        </p>
                    </div>
                </Show>
            </div>
        </AppLayout>
    }
}

fn create_mocks() -> Vec<KanjiCardData> {
    vec![
        KanjiCardData {
            id: "kanji_1".to_string(),
            character: "日".to_string(),
            stroke_count: 4,
            jlpt_level: JlptLevel::N5,
            meanings: vec!["день".to_string(), "солнце".to_string(), "Япония".to_string()],
            onyomi: vec!["ニチ".to_string(), "ジツ".to_string()],
            kunyomi: vec!["ひ".to_string(), "-び".to_string()],
            radicals: vec![
                RadicalInfo {
                    character: "口".to_string(),
                    meaning: "рот".to_string(),
                    strokes: 3,
                },
                RadicalInfo {
                    character: "一".to_string(),
                    meaning: "один".to_string(),
                    strokes: 1,
                },
            ],
            status: CardStatus::New,
            difficulty: 30,
            stability: 50,
            next_review: chrono::Local::now().date_naive(),
            is_in_knowledge_set: false,
        },
        KanjiCardData {
            id: "kanji_2".to_string(),
            character: "本".to_string(),
            stroke_count: 5,
            jlpt_level: JlptLevel::N5,
            meanings: vec!["основа".to_string(), "корень".to_string(), "книга".to_string()],
            onyomi: vec!["ホン".to_string()],
            kunyomi: vec!["もと".to_string()],
            radicals: vec![
                RadicalInfo {
                    character: "木".to_string(),
                    meaning: "дерево".to_string(),
                    strokes: 4,
                },
                RadicalInfo {
                    character: "一".to_string(),
                    meaning: "один".to_string(),
                    strokes: 1,
                },
            ],
            status: CardStatus::InProgress,
            difficulty: 45,
            stability: 65,
            next_review: chrono::Local::now().date_naive() + chrono::Duration::days(3),
            is_in_knowledge_set: true,
        },
        KanjiCardData {
            id: "kanji_3".to_string(),
            character: "人".to_string(),
            stroke_count: 2,
            jlpt_level: JlptLevel::N5,
            meanings: vec!["человек".to_string(), "люди".to_string()],
            onyomi: vec!["ジン".to_string(), "ニン".to_string()],
            kunyomi: vec!["ひと".to_string(), "-り".to_string()],
            radicals: vec![
                RadicalInfo {
                    character: "亻".to_string(),
                    meaning: "человек".to_string(),
                    strokes: 2,
                },
            ],
            status: CardStatus::Mastered,
            difficulty: 20,
            stability: 85,
            next_review: chrono::Local::now().date_naive() + chrono::Duration::days(14),
            is_in_knowledge_set: true,
        },
        KanjiCardData {
            id: "kanji_4".to_string(),
            character: "学".to_string(),
            stroke_count: 8,
            jlpt_level: JlptLevel::N4,
            meanings: vec!["учиться".to_string(), "изучать".to_string()],
            onyomi: vec!["ガク".to_string()],
            kunyomi: vec!["まな".to_string(), "-び".to_string()],
            radicals: vec![
                RadicalInfo {
                    character: "子".to_string(),
                    meaning: "ребенок".to_string(),
                    strokes: 3,
                },
                RadicalInfo {
                    character: "宀".to_string(),
                    meaning: "крыша".to_string(),
                    strokes: 3,
                },
            ],
            status: CardStatus::Difficult,
            difficulty: 75,
            stability: 35,
            next_review: chrono::Local::now().date_naive() + chrono::Duration::days(1),
            is_in_knowledge_set: true,
        },
        KanjiCardData {
            id: "kanji_5".to_string(),
            character: "生".to_string(),
            stroke_count: 5,
            jlpt_level: JlptLevel::N3,
            meanings: vec!["жизнь".to_string(), "рождаться".to_string()],
            onyomi: vec!["セイ".to_string(), "ショウ".to_string()],
            kunyomi: vec!["い.きる".to_string(), "う.まれる".to_string(), "おう".to_string()],
            radicals: vec![
                RadicalInfo {
                    character: "生".to_string(),
                    meaning: "жизнь".to_string(),
                    strokes: 5,
                },
            ],
            status: CardStatus::New,
            difficulty: 55,
            stability: 45,
            next_review: chrono::Local::now().date_naive(),
            is_in_knowledge_set: false,
        },
    ]
}