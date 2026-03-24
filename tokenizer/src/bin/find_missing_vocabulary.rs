use clap::Parser;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tokio::time::sleep;

#[derive(Parser)]
#[command(name = "find_missing_vocabulary")]
#[command(about = "Find vocabulary words from well-known sets missing from dictionary and generate translations", long_about = None)]
struct Cli {
    /// Output path for the markdown report (default: missing_vocabulary.md in project root)
    #[arg(short, long)]
    output: Option<PathBuf>,

    /// Auto-generate missing words with translations
    #[arg(short, long)]
    generate: bool,

    /// OpenAI API base URL (default: http://10.2.11.6:8001/v1)
    #[arg(long, default_value = "http://10.2.11.6:8001/v1")]
    api_base: String,

    /// OpenAI API key (default: none)
    #[arg(long, default_value = "none")]
    api_key: String,

    /// Number of concurrent translation requests
    #[arg(short = 'w', long, default_value = "32")]
    workers: usize,

    /// Chunk size for processing
    #[arg(long, default_value = "512")]
    chunk_size: usize,

    /// Only translate to Russian
    #[arg(long)]
    russian_only: bool,

    /// Only translate to English
    #[arg(long)]
    english_only: bool,
}

#[derive(Debug, Serialize)]
struct ChatMessage {
    role: String,
    content: String,
}

#[derive(Debug, Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<ChatMessage>,
    max_tokens: u32,
    temperature: f32,
    top_p: f32,
    presence_penalty: f32,
}

#[derive(Debug, Deserialize)]
struct ChatResponse {
    choices: Vec<ChatChoice>,
}

#[derive(Debug, Deserialize)]
struct ChatChoice {
    message: ChatMessageContent,
}

#[derive(Debug, Deserialize)]
struct ChatMessageContent {
    content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct VocabularyEntry {
    #[serde(skip_serializing_if = "Option::is_none")]
    russian_translation: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    english_translation: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    found_in_sets: Option<Vec<String>>,
}

fn get_russian_translation_prompt(word: &str) -> String {
    format!(
        r#"<prompt>
  <task>
    Ты — профессиональный лексикограф-японист. Твоя задача: дать точные, минимальные, но исчерпывающие значения японского слова на русском языке.
  </task>

  <word>
    {}
  </word>

  <success_brief>
    <format>
      <description>Строго соблюдай следующий формат вывода (markdown):</description>
      <template>
<![CDATA[
- Перевод 1
- Перевод 2
- Перевод 3

> Комментарий (только при острой необходимости)
]]>
      </template>
    </format>

    <quality_criteria>
      <criterion name="минимализм">Не дублируй смыслы (избегай: "вечер/вечером/вечерний", "стирать/стирка")</criterion>
      <criterion name="разные значения">Каждое значение должно быть семантически отличным от других</criterion>
      <criterion name="язык">ТОЛЬКО русский текст (никакого японского: кандзи, каны, ромадзи, прочтений)</criterion>
      <criterion name="структура">Маркированный список + опционально блок цитаты для комментария</criterion>
      <criterion name="объём">1-5 значений максимум (для многозначных слов), 1-2 для однозначных</criterion>
    </quality_criteria>
  </success_brief>

  <rules>
    <rule id="1">НЕ дублируй грамматические формы одного слова (существительное ≠ глагол от того же корня, если смысл тот же)</rule>
    <rule id="2">НЕ добавляй японский текст в ответ (никаких прочтений, кандзи, примеров на японском)</rule>
    <rule id="3">НЕ пиши вступление, заключение, пояснения перед списком (сразу начинай со списка)</rule>
    <rule id="4">Комментарий в блоке цитаты — только если значение неочевидно или требуется уточнение контекста</rule>
    <rule id="5">Группируй близкие семантически смыслы в одно значение</rule>
    <rule id="6">Если слово имеет омонимы — указывай только основные значения, не все возможные</rule>
    <rule id="7">Приоритет: частотные значения → редкие значения</rule>
  </rules>

  <examples>
    <good>
      <word>かける (kakeru)</word>
      <output>
<![CDATA[
- Вешать
- Тратить (время, деньги)
- Звонить (по телефону)
- Надевать (очки, страховку)
]]>
      </output>
      <reason>4 разных семантических значения одного глагола</reason>
    </good>

    <good>
      <word>重い (omoi)</word>
      <output>
<![CDATA[
- Тяжёлый
- Серьёзный (об ошибке, болезни)

> В зависимости от контекста: физический вес или степень важности
]]>
      </output>
      <reason>2 значения + комментарий для уточнения</reason>
    </good>

    <good>
      <word>冷蔵庫 (reizouko)</word>
      <output>
<![CDATA[
- Холодильник
]]>
      </output>
      <reason>Одно значение, не нужно дублировать</reason>
    </good>

    <bad>
      <word>冷蔵庫 (reizouko)</word>
      <output>
<![CDATA[
- Холодильник
- Морозильная камера
- Холод
- Охлаждение
]]>
      </output>
      <reason>Избыточно: по сути одно устройство, остальные — не значения слова</reason>
    </bad>

    <bad>
      <word>夕べ (yuube)</word>
      <output>
<![CDATA[
- Вечер
- Вечером
- Вечерний
]]>
      </output>
      <reason>Грамматические формы одного смысла, нужно одно значение</reason>
    </bad>
  </examples>

  <conversation>
    <instruction>
      Отвечай только markdown по заданному формату
    </instruction>
  </conversation>
</prompt>"#,
        word
    )
}

fn get_english_translation_prompt(word: &str) -> String {
    format!(
        r#"<prompt>
  <task>
    You are a professional Japanese-English lexicographer. Your task: provide accurate, minimal, but comprehensive meanings of the Japanese word in English.
  </task>

  <word>
    {}
  </word>

  <success_brief>
    <format>
      <description>Strictly follow this output format (markdown):</description>
      <template>
<![CDATA[
- Translation 1
- Translation 2
- Translation 3

> Comment (only when absolutely necessary)
]]>
      </template>
    </format>

    <quality_criteria>
      <criterion name="minimalism">Do not duplicate meanings (avoid: "evening/in the evening/evening (adj)")</criterion>
      <criterion name="different meanings">Each meaning must be semantically distinct</criterion>
      <criterion name="language">ONLY English text (no Japanese: kanji, kana, romaji, readings)</criterion>
      <criterion name="structure">Bulleted list + optional blockquote for comments</criterion>
      <criterion name="volume">1-5 meanings maximum (for polysemous words), 1-2 for monosemous words</criterion>
    </quality_criteria>
  </success_brief>

  <rules>
    <rule id="1">DO NOT duplicate grammatical forms of the same word (noun ≠ verb from the same root if meaning is the same)</rule>
    <rule id="2">DO NOT add Japanese text in the answer (no readings, kanji, examples in Japanese)</rule>
    <rule id="3">DO NOT write introduction, conclusion, explanations before the list (start with the list immediately)</rule>
    <rule id="4">Comment in blockquote — only if meaning is not obvious or context clarification is needed</rule>
    <rule id="5">Group semantically close meanings into one meaning</rule>
    <rule id="6">If word has homonyms — indicate only main meanings, not all possible ones</rule>
    <rule id="7">Priority: frequent meanings → rare meanings</rule>
  </rules>

  <examples>
    <good>
      <word>かける (kakeru)</word>
      <output>
<![CDATA[
- To hang
- To spend (time, money)
- To make a phone call
- To put on (glasses, insurance)
]]>
      </output>
      <reason>4 different semantic meanings of one verb</reason>
    </good>

    <good>
      <word>重い (omoi)</word>
      <output>
<![CDATA[
- Heavy
- Serious (about a mistake, illness)

> Depending on context: physical weight or degree of importance
]]>
      </output>
      <reason>2 meanings + comment for clarification</reason>
    </good>

    <good>
      <word>冷蔵庫 (reizouko)</word>
      <output>
<![CDATA[
- Refrigerator
]]>
      </output>
      <reason>One meaning, no need to duplicate</reason>
    </good>

    <bad>
      <word>冷蔵庫 (reizouko)</word>
      <output>
<![CDATA[
- Refrigerator
- Freezer
- Cold
- Cooling
]]>
      </output>
      <reason>Redundant: essentially one device, the rest are not meanings of the word</reason>
    </bad>

    <bad>
      <word>夕べ (yuube)</word>
      <output>
<![CDATA[
- Evening
- In the evening
- Evening (adj)
]]>
      </output>
      <reason>Grammatical forms of one meaning, need one meaning</reason>
    </bad>
  </examples>

  <conversation>
    <instruction>
      Reply only in markdown according to the specified format
    </instruction>
  </conversation>
</prompt>"#,
        word
    )
}

async fn translate_word(
    word: &str,
    client: &reqwest::Client,
    api_base: &str,
    api_key: &str,
    to_russian: bool,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let prompt = if to_russian {
        get_russian_translation_prompt(word)
    } else {
        get_english_translation_prompt(word)
    };

    let request = ChatRequest {
        model: "llm".to_string(),
        messages: vec![ChatMessage {
            role: "user".to_string(),
            content: prompt,
        }],
        max_tokens: 12144,
        temperature: 0.7,
        top_p: 0.8,
        presence_penalty: 1.5,
    };

    let max_retries = 3;
    for attempt in 0..max_retries {
        let response = client
            .post(format!("{}/chat/completions", api_base))
            .header("Authorization", format!("Bearer {}", api_key))
            .json(&request)
            .timeout(Duration::from_secs(60))
            .send()
            .await;

        match response {
            Ok(resp) if resp.status().is_success() => match resp.json::<ChatResponse>().await {
                Ok(chat_response) => {
                    if let Some(choice) = chat_response.choices.first() {
                        return Ok(choice.message.content.trim().to_string());
                    }
                }
                Err(e) => tracing::warn!("JSON parse error for word '{}': {}", word, e),
            },
            Ok(resp) => {
                tracing::warn!(
                    "API error for word '{}' (attempt {}): HTTP {}",
                    word,
                    attempt + 1,
                    resp.status()
                );
            }
            Err(e) => {
                tracing::warn!(
                    "Request error for word '{}' (attempt {}): {}",
                    word,
                    attempt + 1,
                    e
                );
            }
        }

        if attempt < max_retries - 1 {
            sleep(Duration::from_secs(1)).await;
        }
    }

    Err(format!(
        "Failed to translate word '{}' after {} attempts",
        word, max_retries
    )
    .into())
}

fn get_base_path() -> PathBuf {
    let mut base_path: PathBuf = std::env::var("CARGO_MANIFEST_DIR")
        .unwrap_or_else(|_| ".".to_string())
        .into();

    if base_path.ends_with("tokenizer") {
        base_path.pop();
    }

    base_path
}

fn load_well_known_sets(base_path: &Path) -> HashMap<String, Vec<String>> {
    let mut word_to_sets: HashMap<String, Vec<String>> = HashMap::new();
    let sets_dir = base_path
        .join("origa_ui")
        .join("public")
        .join("domain")
        .join("well_known_set");

    if !sets_dir.exists() {
        tracing::warn!("Directory not found: {}", sets_dir.display());
        return word_to_sets;
    }

    fn collect_json_files(dir: &Path) -> Vec<PathBuf> {
        let mut files = Vec::new();
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    files.extend(collect_json_files(&path));
                } else if path.extension().is_some_and(|ext| ext == "json") {
                    files.push(path);
                }
            }
        }
        files
    }

    for json_file in collect_json_files(&sets_dir) {
        if json_file
            .file_name()
            .is_some_and(|name| name == "well_known_sets_meta.json")
        {
            continue;
        }

        let set_name = json_file
            .file_stem()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        match fs::read_to_string(&json_file) {
            Ok(content) => match serde_json::from_str::<Value>(&content) {
                Ok(data) => {
                    if let Some(words) = data.get("words").and_then(|v| v.as_array()) {
                        for word in words.iter().filter_map(|v| v.as_str()) {
                            let word = word.to_string();
                            let sets = word_to_sets.entry(word).or_default();
                            if !sets.contains(&set_name) {
                                sets.push(set_name.clone());
                            }
                        }
                    }
                }
                Err(e) => {
                    tracing::warn!("Could not parse JSON {}: {}", json_file.display(), e)
                }
            },
            Err(e) => tracing::warn!("Could not read file {}: {}", json_file.display(), e),
        }
    }

    word_to_sets
}

fn load_dictionary(base_path: &Path) -> HashMap<String, VocabularyEntry> {
    let mut dictionary: HashMap<String, VocabularyEntry> = HashMap::new();
    let vocab_dir = base_path
        .join("origa_ui")
        .join("public")
        .join("dictionary")
        .join("vocabulary");

    if !vocab_dir.exists() {
        tracing::warn!("Directory not found: {}", vocab_dir.display());
        return dictionary;
    }

    let mut chunk_files: Vec<PathBuf> = fs::read_dir(&vocab_dir)
        .unwrap_or_else(|_| panic!("Cannot read directory: {}", vocab_dir.display()))
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .map(|name| name.starts_with("chunk_") && name.ends_with(".json"))
                .unwrap_or(false)
        })
        .collect();

    chunk_files.sort();

    for chunk_file in chunk_files {
        match fs::read_to_string(&chunk_file) {
            Ok(content) => match serde_json::from_str::<Value>(&content) {
                Ok(data) => {
                    if let Some(obj) = data.as_object() {
                        for (word, entry_data) in obj {
                            let entry: VocabularyEntry = if entry_data.is_null() {
                                VocabularyEntry {
                                    russian_translation: None,
                                    english_translation: None,
                                    found_in_sets: None,
                                }
                            } else {
                                serde_json::from_value(entry_data.clone()).unwrap_or(
                                    VocabularyEntry {
                                        russian_translation: None,
                                        english_translation: None,
                                        found_in_sets: None,
                                    },
                                )
                            };
                            dictionary.insert(word.clone(), entry);
                        }
                    }
                }
                Err(e) => tracing::warn!("Could not parse JSON {}: {}", chunk_file.display(), e),
            },
            Err(e) => tracing::warn!("Could not read file {}: {}", chunk_file.display(), e),
        }
    }

    dictionary
}

fn find_missing_words(
    word_to_sets: &HashMap<String, Vec<String>>,
    dictionary: &HashMap<String, VocabularyEntry>,
) -> HashMap<String, Vec<String>> {
    word_to_sets
        .iter()
        .filter(|(word, _)| !dictionary.contains_key(*word))
        .map(|(word, sets)| (word.clone(), sets.clone()))
        .collect()
}

fn generate_report(
    missing_words: &HashMap<String, Vec<String>>,
    total_set_words: usize,
    dict_words: usize,
    output_path: &Path,
) {
    let mut sorted_missing: Vec<_> = missing_words.iter().collect();
    sorted_missing.sort_by(|a, b| match b.1.len().cmp(&a.1.len()) {
        std::cmp::Ordering::Equal => a.0.cmp(b.0),
        other => other,
    });

    let mut lines = Vec::new();

    lines.push("# Missing Vocabulary".to_string());
    lines.push(String::new());
    lines.push("## Statistics".to_string());
    lines.push(format!("- Total unique words in sets: {}", total_set_words));
    lines.push(format!("- Words in dictionary: {}", dict_words));
    lines.push(format!("- Missing words: {}", missing_words.len()));
    lines.push(String::new());
    lines.push("## Missing Words List".to_string());
    lines.push(String::new());
    lines.push("| Word | Found in sets |".to_string());
    lines.push("|------|---------------|".to_string());

    for (word, sets) in &sorted_missing {
        let mut sorted_sets = (*sets).clone();
        sorted_sets.sort();
        let sets_str = sorted_sets.join(", ");
        lines.push(format!("| {} | {} |", word, sets_str));
    }

    if sorted_missing.is_empty() {
        lines.push("_No missing words found._".to_string());
    }

    lines.push(String::new());
    lines.push("## Plain List (for copy-paste)".to_string());
    lines.push(String::new());

    for (word, _) in &sorted_missing {
        lines.push((*word).clone());
    }

    match fs::write(output_path, lines.join("\n")) {
        Ok(_) => tracing::info!("Report saved to: {}", output_path.display()),
        Err(e) => tracing::error!("Failed to write report: {}", e),
    }
}

async fn generate_missing_vocabulary(
    missing_words: &HashMap<String, Vec<String>>,
    dictionary: HashMap<String, VocabularyEntry>,
    client: &reqwest::Client,
    cli: &Cli,
    base_path: &Path,
) {
    let words: Vec<String> = missing_words.keys().cloned().collect();
    let total_words = words.len();

    if total_words == 0 {
        tracing::info!("No missing words to generate");
        return;
    }

    tracing::info!(
        "Starting translation of {} missing words with {} workers...",
        total_words,
        cli.workers
    );

    let dictionary = Arc::new(Mutex::new(dictionary));
    let completed = Arc::new(Mutex::new(0usize));
    let errors = Arc::new(Mutex::new(0usize));

    let chunk_size = cli.chunk_size;
    let total_chunks = total_words.div_ceil(chunk_size);

    for (chunk_num, chunk_start) in (0..total_words).step_by(chunk_size).enumerate() {
        let chunk_end = (chunk_start + chunk_size).min(total_words);
        let chunk: Vec<String> = words[chunk_start..chunk_end].to_vec();

        tracing::info!(
            "\n--- Chunk {}/{} ({} to {}) ---",
            chunk_num + 1,
            total_chunks,
            chunk_start,
            chunk_end
        );

        let mut tasks = Vec::new();

        for word in chunk {
            let client = client.clone();
            let api_base = cli.api_base.clone();
            let api_key = cli.api_key.clone();
            let dictionary = Arc::clone(&dictionary);
            let completed = Arc::clone(&completed);
            let errors = Arc::clone(&errors);
            let russian_only = cli.russian_only;
            let english_only = cli.english_only;
            let found_in_sets = missing_words.get(&word).cloned();

            let handle = tokio::spawn(async move {
                let mut russian_translation = None;
                let mut english_translation = None;

                // Translate to Russian (unless english_only is set)
                if !english_only {
                    match translate_word(&word, &client, &api_base, &api_key, true).await {
                        Ok(translation) => russian_translation = Some(translation),
                        Err(e) => {
                            tracing::warn!("Russian translation error for '{}': {}", word, e);
                            let mut err = errors.lock().await;
                            *err += 1;
                            return;
                        }
                    }
                }

                // Translate to English (unless russian_only is set)
                if !russian_only {
                    match translate_word(&word, &client, &api_base, &api_key, false).await {
                        Ok(translation) => english_translation = Some(translation),
                        Err(e) => {
                            tracing::warn!("English translation error for '{}': {}", word, e);
                            let mut err = errors.lock().await;
                            *err += 1;
                            return;
                        }
                    }
                }

                // Save to dictionary
                let entry = VocabularyEntry {
                    russian_translation,
                    english_translation,
                    found_in_sets,
                };

                let mut dict = dictionary.lock().await;
                dict.insert(word.clone(), entry);

                let mut comp = completed.lock().await;
                *comp += 1;

                if *comp % 50 == 0 || *comp == total_words {
                    let percent = (*comp as f64 / total_words as f64) * 100.0;
                    print!("\r  Progress: {}/{} ({:.1}%)", *comp, total_words, percent);
                }
            });

            tasks.push(handle);

            // Limit concurrent tasks
            if tasks.len() >= cli.workers {
                futures::future::join_all(tasks.drain(..)).await;
            }
        }

        // Wait for remaining tasks
        futures::future::join_all(tasks).await;

        // Save dictionary after each chunk
        let dict = dictionary.lock().await;
        save_dictionary(&dict, base_path);
    }

    let comp = completed.lock().await;
    let err = errors.lock().await;
    tracing::info!(
        "\n\nTranslation complete: {} words processed, {} errors",
        *comp,
        *err
    );
}

fn save_dictionary(dictionary: &HashMap<String, VocabularyEntry>, base_path: &Path) {
    let vocab_dir = base_path
        .join("origa_ui")
        .join("public")
        .join("dictionary")
        .join("vocabulary");

    // Create a new chunk file for the missing words
    let chunk_files: Vec<PathBuf> = fs::read_dir(&vocab_dir)
        .unwrap_or_else(|_| panic!("Cannot read directory: {}", vocab_dir.display()))
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .map(|name| name.starts_with("chunk_") && name.ends_with(".json"))
                .unwrap_or(false)
        })
        .collect();

    // Find the next chunk number
    let next_chunk = chunk_files
        .iter()
        .filter_map(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .and_then(|name| {
                    name.trim_start_matches("chunk_")
                        .trim_end_matches(".json")
                        .parse::<usize>()
                        .ok()
                })
        })
        .max()
        .map(|n| n + 1)
        .unwrap_or(0);

    // Save to a new chunk file
    let chunk_path = vocab_dir.join(format!("chunk_{:02}.json", next_chunk));
    match serde_json::to_string_pretty(&dictionary) {
        Ok(json) => {
            if let Err(e) = fs::write(&chunk_path, json) {
                tracing::error!("Failed to write chunk file {}: {}", chunk_path.display(), e);
            } else {
                tracing::info!("Saved dictionary to {}", chunk_path.display());
            }
        }
        Err(e) => tracing::error!("Failed to serialize dictionary: {}", e),
    }
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let cli = Cli::parse();
    let base_path = get_base_path();
    let output_path = cli
        .output
        .clone()
        .unwrap_or_else(|| base_path.join("missing_vocabulary.md"));

    tracing::info!("Loading well-known sets...");
    let word_to_sets = load_well_known_sets(&base_path);
    let total_set_words = word_to_sets.len();

    tracing::info!("Loading dictionary...");
    let dictionary = load_dictionary(&base_path);

    tracing::info!("Finding missing words...");
    let missing_words = find_missing_words(&word_to_sets, &dictionary);

    tracing::info!("Generating report...");
    generate_report(
        &missing_words,
        total_set_words,
        dictionary.len(),
        &output_path,
    );

    tracing::info!(
        "\nSummary:\n  Unique words in sets: {}\n  Dictionary words: {}\n  Missing words: {}",
        total_set_words,
        dictionary.len(),
        missing_words.len()
    );

    if cli.generate && !missing_words.is_empty() {
        let client = reqwest::Client::new();
        generate_missing_vocabulary(&missing_words, dictionary, &client, &cli, &base_path).await;
    } else if !missing_words.is_empty() {
        tracing::info!("\nTo generate missing vocabulary, run with --generate flag");
    }
}
