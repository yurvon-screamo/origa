use clap::Parser;
use flate2::read::DeflateDecoder;
use origa::domain::{
    DictionaryData, OrigaError, init_dictionary, is_dictionary_loaded, tokenize_text,
};
use std::collections::HashSet;
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};

#[derive(Parser)]
#[command(name = "tokenizer")]
#[command(about = "Японский токенизатор на основе Lindera", long_about = None)]
struct Cli {
    /// Текст для токенизации или путь к файлу
    text: String,

    /// Читать текст из файла
    #[arg(short, long)]
    file: bool,
}

fn decompress(data: Vec<u8>) -> Result<Vec<u8>, OrigaError> {
    let mut decoder = DeflateDecoder::new(&data[..]);
    let mut decompressed = Vec::new();
    decoder
        .read_to_end(&mut decompressed)
        .map_err(|e| OrigaError::TokenizerError {
            reason: format!("Decompression failed: {}", e),
        })?;
    Ok(decompressed)
}

pub fn load_dictionary() -> Result<(), OrigaError> {
    if is_dictionary_loaded() {
        return Ok(());
    }

    // Attempt to find dictionary in origa_ui/public/dictionaries/unidic
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".to_string());
    let mut dict_dir = PathBuf::from(manifest_dir);

    // If we're in tokenizer/ directory, go up to root
    if dict_dir.ends_with("tokenizer") {
        dict_dir.pop();
    }

    dict_dir = dict_dir
        .join("origa_ui")
        .join("public")
        .join("dictionaries")
        .join("unidic");

    if !dict_dir.exists() {
        // Fallback for when running from root or other dirs
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

    let data = DictionaryData {
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

fn main() {
    let cli = Cli::parse();

    if let Err(e) = load_dictionary() {
        eprintln!("Ошибка загрузки словаря: {}", e);
        std::process::exit(1);
    }

    let mut vocab_words: HashSet<String> = HashSet::new();

    if cli.file || Path::new(&cli.text).exists() {
        let bytes = std::fs::read(&cli.text).unwrap_or_else(|e| {
            eprintln!("Ошибка чтения файла {}: {}", cli.text, e);
            std::process::exit(1);
        });

        let text = String::from_utf8_lossy(&bytes);

        for line in text.lines() {
            let result = tokenize_text(line).map(|tokens| {
                for token in tokens {
                    if token.part_of_speech().is_vocabulary_word() {
                        vocab_words.insert(token.orthographic_base_form().to_string());
                    }
                }
            });

            if let Err(e) = result {
                eprintln!("Ошибка токенизации: {}", e);
                std::process::exit(1);
            }
        }
    } else {
        match tokenize_text(&cli.text) {
            Ok(tokens) => {
                for token in tokens {
                    if token.part_of_speech().is_vocabulary_word() {
                        vocab_words.insert(token.orthographic_base_form().to_string());
                    }
                }
            }
            Err(e) => {
                eprintln!("Ошибка токенизации: {}", e);
                std::process::exit(1);
            }
        }
    }

    let mut sorted_words: Vec<String> = vocab_words.into_iter().collect();
    sorted_words.sort();

    println!("{}", sorted_words.join(" "));
}
