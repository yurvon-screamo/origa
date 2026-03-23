use clap::Parser;
use origa::domain::{OrigaError, init_dictionary, tokenize_text};
use serde_json::{Map, Value};
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Parser)]
#[command(name = "tokenize_well_known")]
#[command(about = "Пакетная обработка JSON файлов well_known_set", long_about = None)]
struct Cli {
    /// Путь к директории или JSON файлу
    path: PathBuf,
}

fn load_dictionary() -> Result<(), OrigaError> {
    if origa::domain::is_dictionary_loaded() {
        return Ok(());
    }

    let mut dict_dir: PathBuf = std::env::var("CARGO_MANIFEST_DIR")
        .unwrap_or_else(|_| ".".to_string())
        .into();

    if dict_dir.ends_with("tokenizer") {
        dict_dir.pop();
    }

    dict_dir = dict_dir
        .join("origa_ui")
        .join("public")
        .join("dictionaries")
        .join("unidic");

    if !dict_dir.exists() {
        dict_dir = PathBuf::from("origa_ui/public/dictionaries/unidic");
    }

    if !dict_dir.exists() {
        return Err(OrigaError::TokenizerError {
            reason: format!("Dictionary directory not found: {}", dict_dir.display()),
        });
    }

    let read_file = |name: &str| -> Result<Vec<u8>, OrigaError> {
        fs::read(dict_dir.join(name)).map_err(|e| OrigaError::TokenizerError {
            reason: format!("Failed to read {}: {}", name, e),
        })
    };

    let decompress = |data: Vec<u8>| -> Result<Vec<u8>, OrigaError> {
        use flate2::read::DeflateDecoder;
        use std::io::Read;
        let mut decoder = DeflateDecoder::new(&data[..]);
        let mut decompressed = Vec::new();
        decoder
            .read_to_end(&mut decompressed)
            .map_err(|e| OrigaError::TokenizerError {
                reason: format!("Decompression failed: {}", e),
            })?;
        Ok(decompressed)
    };

    let data = origa::domain::DictionaryData {
        char_def: decompress(read_file("char_def.bin")?)?,
        matrix: decompress(read_file("matrix.mtx")?)?,
        dict_da: decompress(read_file("dict.da")?)?,
        dict_vals: decompress(read_file("dict.vals")?)?,
        unk: decompress(read_file("unk.bin")?)?,
        words_idx: decompress(read_file("dict.wordsidx")?)?,
        words: decompress(read_file("dict.words")?)?,
        metadata: read_file("metadata.json")?,
    };

    init_dictionary(data)
}

fn tokenize_words(words: &[String]) -> Result<Vec<String>, OrigaError> {
    let text = words.join(" ");
    let tokens = tokenize_text(&text)?;

    let mut unique_words: HashSet<String> = HashSet::new();
    for token in tokens {
        if token.part_of_speech().is_vocabulary_word() {
            unique_words.insert(token.orthographic_base_form().to_string());
        }
    }

    let mut sorted: Vec<String> = unique_words.into_iter().collect();
    sorted.sort();
    Ok(sorted)
}

fn process_file(path: &Path) -> Result<(), OrigaError> {
    let content = fs::read_to_string(path).map_err(|e| OrigaError::TokenizerError {
        reason: format!("Failed to read {}: {}", path.display(), e),
    })?;

    let mut json: Map<String, Value> =
        serde_json::from_str(&content).map_err(|e| OrigaError::TokenizerError {
            reason: format!("Failed to parse {}: {}", path.display(), e),
        })?;

    let words = json
        .get("words")
        .and_then(|v| v.as_array())
        .ok_or_else(|| OrigaError::TokenizerError {
            reason: format!("No 'words' array in {}", path.display()),
        })?;

    let words_vec: Vec<String> = words
        .iter()
        .filter_map(|v| v.as_str().map(String::from))
        .collect();

    let new_words = tokenize_words(&words_vec)?;

    json.insert(
        "words".to_string(),
        Value::Array(new_words.into_iter().map(Value::String).collect()),
    );

    let output = serde_json::to_string_pretty(&Value::Object(json)).map_err(|e| {
        OrigaError::TokenizerError {
            reason: format!("Failed to serialize {}: {}", path.display(), e),
        }
    })?;

    fs::write(path, output).map_err(|e| OrigaError::TokenizerError {
        reason: format!("Failed to write {}: {}", path.display(), e),
    })?;

    tracing::info!("Processed: {}", path.display());
    Ok(())
}

fn collect_json_files(path: &Path) -> Result<Vec<PathBuf>, OrigaError> {
    let mut files = Vec::new();

    if path.is_file() {
        if path.extension().is_some_and(|ext| ext == "json") {
            files.push(path.to_path_buf());
        }
    } else if path.is_dir() {
        for entry in fs::read_dir(path).map_err(|e| OrigaError::TokenizerError {
            reason: format!("Failed to read dir {}: {}", path.display(), e),
        })? {
            let entry = entry.map_err(|e| OrigaError::TokenizerError {
                reason: format!("Failed to read entry: {}", e),
            })?;
            let entry_path = entry.path();

            if entry_path.is_dir() {
                files.extend(collect_json_files(&entry_path)?);
            } else if entry_path.extension().is_some_and(|ext| ext == "json") {
                files.push(entry_path);
            }
        }
    }

    Ok(files)
}

fn main() {
    tracing_subscriber::fmt::init();

    let cli = Cli::parse();

    if let Err(e) = load_dictionary() {
        eprintln!("Ошибка загрузки словаря: {}", e);
        std::process::exit(1);
    }

    let files = match collect_json_files(&cli.path) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("Ошибка сбора файлов: {}", e);
            std::process::exit(1);
        }
    };

    if files.is_empty() {
        eprintln!("JSON файлы не найдены");
        std::process::exit(1);
    }

    tracing::info!("Found {} JSON file(s)", files.len());

    for file in files {
        if let Err(e) = process_file(&file) {
            eprintln!("Ошибка обработки {}: {}", file.display(), e);
        }
    }
}
