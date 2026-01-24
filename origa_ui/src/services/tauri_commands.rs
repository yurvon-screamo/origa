use leptos::*;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"])]
    async fn invoke(cmd: &str, args: JsValue) -> JsValue;

    #[wasm_bindgen(js_namespace = ["window"], thread_local_v2)]
    static __TAURI__: JsValue;
}

fn is_tauri_available() -> bool {
    __TAURI__.with(|tauri| !tauri.is_undefined())
}

pub async fn invoke_tauri_command<T: serde::de::DeserializeOwned>(
    command: &str,
    args: impl serde::Serialize,
) -> Result<T, String> {
    if !is_tauri_available() {
        return get_mock_data(command);
    }

    let args_js = serde_wasm_bindgen::to_value(&args)
        .map_err(|e| format!("Failed to serialize args: {}", e))?;

    let result = invoke(command, args_js).await;
    serde_wasm_bindgen::from_value(result)
        .map_err(|e| format!("Failed to deserialize result: {}", e))
}

fn get_mock_data<T: serde::de::DeserializeOwned>(command: &str) -> Result<T, String> {
    match command {
        "get_user_info" => {
            let mock_user = UserInfo {
                id: "mock_user_123".to_string(),
                name: "Пользователь".to_string(),
                study_streak: 5,
                cards_learned: 42,
            };
            serde_wasm_bindgen::from_value(serde_wasm_bindgen::to_value(&mock_user).unwrap())
                .map_err(|e| format!("Mock data error: {}", e))
        }
        "select_cards_to_lesson" => {
            let mock_cards = vec![
                LessonCard {
                    id: "card1".to_string(),
                    question: "こんにちは".to_string(),
                    answer: "Привет".to_string(),
                    furigana: None,
                    example: Some(ExampleData {
                        text: "こんにちは、元気ですか？".to_string(),
                        translation: "Привет, как дела?".to_string(),
                    }),
                },
                LessonCard {
                    id: "card2".to_string(),
                    question: "ありがとう".to_string(),
                    answer: "Спасибо".to_string(),
                    furigana: None,
                    example: None,
                },
            ];
            serde_wasm_bindgen::from_value(serde_wasm_bindgen::to_value(&mock_cards).unwrap())
                .map_err(|e| format!("Mock data error: {}", e))
        }
        "get_vocabulary_cards" => {
            let mock_vocab = vec![VocabularyCard {
                id: "vocab1".to_string(),
                word: "学校".to_string(),
                reading: "がっこう".to_string(),
                meaning: "школа".to_string(),
                difficulty: 3,
                status: "learning".to_string(),
            }];
            serde_wasm_bindgen::from_value(serde_wasm_bindgen::to_value(&mock_vocab).unwrap())
                .map_err(|e| format!("Mock data error: {}", e))
        }
        "get_kanji_list" => {
            let mock_kanji = vec![KanjiInfo {
                character: "学".to_string(),
                readings: vec!["がく".to_string(), "まな".to_string()],
                meanings: vec!["учиться".to_string(), "изучать".to_string()],
                strokes: 8,
                jlpt: "N5".to_string(),
                added: false,
            }];
            serde_wasm_bindgen::from_value(serde_wasm_bindgen::to_value(&mock_kanji).unwrap())
                .map_err(|e| format!("Mock data error: {}", e))
        }
        _ => Err(format!("Mock for command '{}' not implemented", command)),
    }
}

// User info types
#[derive(Serialize)]
pub struct GetUserInfoArgs {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserInfo {
    pub id: String,
    pub name: String,
    pub study_streak: i32,
    pub cards_learned: i32,
}

pub async fn get_user_info() -> Result<UserInfo, String> {
    invoke_tauri_command("get_user_info", GetUserInfoArgs {}).await
}

// Lesson card types
#[derive(Serialize)]
pub struct SelectCardsToLessonArgs {
    pub count: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExampleData {
    pub text: String,
    pub translation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LessonCard {
    pub id: String,
    pub question: String,
    pub answer: String,
    pub furigana: Option<String>,
    pub example: Option<ExampleData>,
}

pub async fn select_cards_to_lesson(count: i32) -> Result<Vec<LessonCard>, String> {
    invoke_tauri_command("select_cards_to_lesson", SelectCardsToLessonArgs { count }).await
}

// Rate card
#[derive(Serialize)]
pub struct RateCardArgs {
    pub card_id: String,
    pub rating: i32,
}

pub async fn rate_card(card_id: String, rating: i32) -> Result<(), String> {
    invoke_tauri_command("rate_card", RateCardArgs { card_id, rating }).await
}

// Vocabulary cards
#[derive(Serialize)]
pub struct GetVocabularyCardsArgs {
    pub filter: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VocabularyCard {
    pub id: String,
    pub word: String,
    pub reading: String,
    pub meaning: String,
    pub difficulty: i32,
    pub status: String,
}

pub async fn get_vocabulary_cards(filter: String) -> Result<Vec<VocabularyCard>, String> {
    invoke_tauri_command("get_vocabulary_cards", GetVocabularyCardsArgs { filter }).await
}

// Kanji list
#[derive(Serialize)]
pub struct GetKanjiListArgs {
    pub jlpt_level: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KanjiInfo {
    pub character: String,
    pub readings: Vec<String>,
    pub meanings: Vec<String>,
    pub strokes: i32,
    pub jlpt: String,
    pub added: bool,
}

pub async fn get_kanji_list(jlpt_level: String) -> Result<Vec<KanjiInfo>, String> {
    invoke_tauri_command("get_kanji_list", GetKanjiListArgs { jlpt_level }).await
}

// Import functions
#[derive(Serialize)]
pub struct ImportAnkiFileArgs {
    pub file_path: String,
}

pub async fn import_anki_file(file_path: String) -> Result<i32, String> {
    invoke_tauri_command("import_anki_file", ImportAnkiFileArgs { file_path }).await
}

pub async fn import_duolingo_data() -> Result<i32, String> {
    invoke_tauri_command("import_duolingo_data", serde_json::json!({})).await
}

// Settings
#[derive(Serialize, Deserialize)]
pub struct UserSettings {
    pub llm_provider: String,
    pub api_key: String,
    pub duolingo_jwt: String,
}

pub async fn update_settings(settings: UserSettings) -> Result<(), String> {
    invoke_tauri_command("update_settings", settings).await
}
