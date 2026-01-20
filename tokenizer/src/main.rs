use clap::Parser;
use origa::domain::tokenize_text;
use std::collections::HashSet;
use std::path::Path;

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

fn main() {
    let cli = Cli::parse();

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

            if result.is_err() {
                eprintln!("Ошибка токенизации: {}", result.err().unwrap());
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
