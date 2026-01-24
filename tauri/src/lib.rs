// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

// Origa-specific commands with real use case integration
use std::sync::Arc;
use origa::domain::user::User;
use origa::domain::knowledge::Card;
use origa::domain::memory::{Rating, ReviewInfo};
use origa::domain::value_objects::{Question, Answer, JapaneseLevel, NativeLanguage};
use origa::application::{UserRepository, GetUserInfoUseCase, SelectCardsToLessonUseCase, RateCardUseCase};
use origa::application::error::OrigaError;
use ulid::Ulid;
use tokio::sync::Mutex;
use chrono::Utc;

// Mock repository for now - will be connected to real database later
struct MockUserRepository {
    users: Arc<Mutex<Vec<User>>>,
    cards: Arc<Mutex<Vec<Card>>>,
}

impl MockUserRepository {
    fn new() -> Self {
        // Create user using the actual constructor
        use origa::domain::value_objects::{JapaneseLevel, NativeLanguage};
        let user = User::new(
            "Test User".to_string(),
            JapaneseLevel::N5,
            NativeLanguage::Russian,
        );
        
        let cards = vec![
            Card::new(
                Question::new("日本語".to_string()).unwrap(),
                Answer::new("Японский язык".to_string()).unwrap(),
            ),
            Card::new(
                Question::new("こんにちは".to_string()).unwrap(),
                Answer::new("Здравствуйте".to_string()).unwrap(),
            ),
        ];
        
        Self {
            users: Arc::new(Mutex::new(vec![user])),
            cards: Arc::new(Mutex::new(cards)),
}
}

#[tauri::command]
async fn get_user_info() -> Result<serde_json::Value, String> {
    let user_id = Ulid::new(); // For demo purposes
    let use_case = GetUserInfoUseCase::new(REPOSITORY.as_ref());
    
    match use_case.execute(user_id).await {
        Ok(user) => Ok(serde_json::json!({
            "id": user.id().to_string(),
            "username": user.username(),
            "current_japanese_level": format!("{:?}", user.current_japanese_level()),
            "native_language": format!("{:?}", user.native_language())
        })),
        Err(e) => Err(e.to_string())
    }
}

#[tauri::command]
async fn rate_card(card_id: String, rating: i32) -> Result<(), String> {
    let card_ulid = Ulid::from_string(&card_id).map_err(|e| format!("Invalid card ID: {}", e))?;
    let user_id = Ulid::new(); // For demo purposes
    
    let use_case = RateCardUseCase::new(REPOSITORY.as_ref());
    
    use origa::domain::memory::Rating;
    let rating = match rating {
        1 => Rating::Again,
        2 => Rating::Hard, 
        3 => Rating::Good,
        4 => Rating::Easy,
        _ => Rating::Good, // Default
    };
    
    match use_case.execute(user_id, card_ulid, rating).await {
        Ok(_) => Ok(()),
        Err(e) => Err(e.to_string())
    }
}

#[tauri::command]
async fn get_vocabulary_cards(filter: String) -> Result<Vec<serde_json::Value>, String> {
    let cards = REPOSITORY.cards.lock().await;
    let cards_json: Result<Vec<_>, _> = cards.iter().map(|card| {
        serde_json::json!({
            "id": card.id.to_string(),
            "word": card.question.content(),
            "reading": card.answer.content(), // Simplified for demo
            "meaning": "Перевод", // Simplified for demo
            "difficulty": 1,
            "status": if card.reviews.is_empty() { "new" } else { "learning" }
        })
    }).collect();
    cards_json.map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_kanji_list(jlpt_level: String) -> Result<Vec<serde_json::Value>, String> {
    // Mock kanji data
    let kanji_data = match jlpt_level.as_str() {
        "n5" => vec![
            ("日", vec!["にち", "じつ"], vec!["день", "солнце"], 4),
            ("本", vec!["ほん", "もと"], vec!["книга", "основание"], 5),
            ("人", vec!["ひと", "にん"], vec!["человек"], 2),
        ],
        _ => vec![]
    };
    
    let kanji_json: Vec<_> = kanji_data.into_iter().map(|(char, readings, meanings, strokes)| {
        serde_json::json!({
            "character": char,
            "readings": readings,
            "meanings": meanings,
            "strokes": strokes,
            "jlpt": jlpt_level.to_uppercase(),
            "added": false
        })
    }).collect();
    
    Ok(kanji_json)
}

#[tauri::command]
async fn import_anki_file(_file_path: String) -> Result<i32, String> {
    // Mock implementation
    Ok(10) // Return number of imported cards
}

#[tauri::command]
async fn import_duolingo_data() -> Result<i32, String> {
    // Mock implementation
    Ok(5) // Return number of imported cards
}

#[tauri::command]
async fn update_settings(settings: serde_json::Value) -> Result<(), String> {
    println!("Updating settings: {:?}", settings);
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            greet,
            get_user_info,
            select_cards_to_lesson,
            rate_card,
            get_vocabulary_cards,
            get_kanji_list,
            import_anki_file,
            import_duolingo_data,
            update_settings
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
}