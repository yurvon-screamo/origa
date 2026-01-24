// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

// Origa-specific commands with real use case integration
use origa::application::srs_service::RateMode;
use origa::application::{
    GetUserInfoUseCase, RateCardUseCase, SelectCardsToLessonUseCase, UserRepository,
};
use origa::domain::{JapaneseLevel, NativeLanguage, Rating, User};
use origa::settings::ApplicationEnvironment;
use ulid::Ulid;

#[tauri::command]
async fn get_user_info() -> Result<serde_json::Value, String> {
    let env = ApplicationEnvironment::get();
    let repository = env.get_repository().await.map_err(|e| e.to_string())?;

    // Get first user for demo purposes - create one if none exists
    let users = repository.list().await.map_err(|e| e.to_string())?;
    let user_id = if let Some(user) = users.first() {
        user.id()
    } else {
        // Create a demo user if none exists
        let user = User::new(
            "Demo User".to_string(),
            JapaneseLevel::N5,
            NativeLanguage::Russian,
        );
        let user_id = user.id();
        repository.save(&user).await.map_err(|e| e.to_string())?;
        user_id
    };

    let use_case = GetUserInfoUseCase::new(repository);
    match use_case.execute(user_id).await {
        Ok(user) => Ok(serde_json::json!({
            "id": user.id.to_string(),
            "name": user.username,
            "study_streak": 7, // Mock for now
            "cards_learned": 25, // Mock for now
        })),
        Err(e) => Err(e.to_string()),
    }
}

#[tauri::command]
async fn select_cards_to_lesson(count: i32) -> Result<Vec<serde_json::Value>, String> {
    let env = ApplicationEnvironment::get();
    let repository = env.get_repository().await.map_err(|e| e.to_string())?;

    // Get first user for demo purposes
    let users = repository.list().await.map_err(|e| e.to_string())?;
    let user_id = users.first().map(|u| u.id()).ok_or("No users found")?;

    let use_case = SelectCardsToLessonUseCase::new(repository);
    match use_case.execute(user_id).await {
        Ok(cards_map) => {
            let lesson_cards: Vec<_> = cards_map
                .iter()
                .take(count as usize)
                .map(|(card_id, card)| {
                    serde_json::json!({
                        "id": card_id.to_string(),
                        "question": card.question().text(),
                        "answer": card.answer().text(),
                        "furigana": serde_json::Value::Null, // Will be implemented later
                        "example": serde_json::Value::Null, // Will be implemented later
                    })
                })
                .collect();
            Ok(lesson_cards)
        }
        Err(e) => Err(e.to_string()),
    }
}

#[tauri::command]
async fn rate_card(card_id: String, rating: i32) -> Result<(), String> {
    let env = ApplicationEnvironment::get();
    let repository = env.get_repository().await.map_err(|e| e.to_string())?;
    let srs_service = env.get_srs_service().await.map_err(|e| e.to_string())?;

    let card_ulid = Ulid::from_string(&card_id).map_err(|e| format!("Invalid card ID: {}", e))?;

    // Get first user for demo purposes
    let users = repository.list().await.map_err(|e| e.to_string())?;
    let user_id = users.first().map(|u| u.id()).ok_or("No users found")?;

    let rating_enum = match rating {
        1 => Rating::Again,
        2 => Rating::Hard,
        3 => Rating::Good,
        4 => Rating::Easy,
        _ => Rating::Good, // Default
    };

    let use_case = RateCardUseCase::new(repository, srs_service);
    match use_case
        .execute(user_id, card_ulid, RateMode::StandardLesson, rating_enum)
        .await
    {
        Ok(_) => Ok(()),
        Err(e) => Err(e.to_string()),
    }
}

#[tauri::command]
async fn get_vocabulary_cards(_filter: String) -> Result<Vec<serde_json::Value>, String> {
    let env = ApplicationEnvironment::get();
    let repository = env.get_repository().await.map_err(|e| e.to_string())?;

    // Get first user for demo purposes
    let users = repository.list().await.map_err(|e| e.to_string())?;
    let user_id = users.first().map(|u| u.id()).ok_or("No users found")?;

    // Get user to access their cards
    let user = repository
        .find_by_id(user_id)
        .await
        .map_err(|e| e.to_string())?
        .ok_or("User not found")?;

    let knowledge_set = user.knowledge_set();
    let cards_map = knowledge_set.cards_to_lesson(user.native_language());

    let cards_json: Vec<_> = cards_map
        .iter()
        .take(20) // Limit to 20 for performance
        .map(|(card_id, card)| {
            serde_json::json!({
                "id": card_id.to_string(),
                "word": card.question().text(),
                "reading": card.answer().text(), // Simplified for demo
                "meaning": card.answer().text(), // Simplified for demo
                "difficulty": 1, // Will be calculated later
                "status": "new" // Simplified since we don't have StudyCard here
            })
        })
        .collect();
    Ok(cards_json)
}

#[tauri::command]
async fn get_kanji_list(jlpt_level: String) -> Result<Vec<serde_json::Value>, String> {
    // Mock kanji data for now - will be implemented with real data later
    let kanji_data = match jlpt_level.as_str() {
        "n5" => vec![
            ("日", vec!["にち", "じつ"], vec!["день", "солнце"], 4),
            ("本", vec!["ほん", "もと"], vec!["книга", "основание"], 5),
            ("人", vec!["ひと", "にん"], vec!["человек"], 2),
            ("語", vec!["ご", "かた"], vec!["язык", "слово"], 14),
            ("学", vec!["がく", "まな"], vec!["учиться"], 8),
        ],
        "n4" => vec![
            ("時", vec!["じ", "とき"], vec!["время"], 10),
            (
                "間",
                vec!["あいだ", "かん"],
                vec!["между", "промежуток"],
                12,
            ),
            ("今", vec!["いま", "коん"], vec!["сейчас"], 4),
        ],
        _ => vec![],
    };

    let kanji_json: Vec<_> = kanji_data
        .into_iter()
        .map(|(char, readings, meanings, strokes)| {
            serde_json::json!({
                "character": char,
                "readings": readings,
                "meanings": meanings,
                "strokes": strokes,
                "jlpt": jlpt_level.to_uppercase(),
                "added": false
            })
        })
        .collect();

    Ok(kanji_json)
}

#[tauri::command]
async fn import_anki_file(_file_path: String) -> Result<i32, String> {
    // Mock implementation for now - will be implemented later
    Ok(10) // Return number of imported cards
}

#[tauri::command]
async fn import_duolingo_data() -> Result<i32, String> {
    // Mock implementation for now - will be implemented later
    Ok(5) // Return number of imported cards
}

#[tauri::command]
async fn update_settings(settings: serde_json::Value) -> Result<(), String> {
    println!("Updating settings: {:?}", settings);
    // Will be implemented later with real settings update
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
