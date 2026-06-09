use std::env;
use std::fs;
use std::path::Path;

fn main() {
    build_user_dictionary();
}

fn build_user_dictionary() {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();

    let dict_dir = Path::new(&manifest_dir)
        .join("..")
        .join("cdn")
        .join("dictionaries");

    let metadata_path = dict_dir.join("metadata.json");
    let csv_path = Path::new(&manifest_dir)
        .join("src")
        .join("domain")
        .join("tokenizer")
        .join("user_dictionary.csv");

    if !metadata_path.exists() || !csv_path.exists() {
        println!(
            "cargo:warning=Skipping user dictionary build: metadata or CSV not found"
        );
        return;
    }

    println!("cargo:rerun-if-changed=src/domain/tokenizer/user_dictionary.csv");
    println!("cargo:rerun-if-changed=../cdn/dictionaries/metadata.json");

    let metadata_bytes = fs::read(&metadata_path).expect("Failed to read metadata.json");
    let metadata: lindera_dictionary::dictionary::metadata::Metadata =
        serde_json::from_slice(&metadata_bytes).expect("Failed to parse metadata.json");

    let builder = lindera_dictionary::builder::DictionaryBuilder::new(metadata);
    let user_dict = builder
        .build_user_dict(&csv_path)
        .expect("Failed to build user dictionary from CSV");

    let out_dir = env::var("OUT_DIR").unwrap();
    let out_path = Path::new(&out_dir).join("user_dictionary.bin");

    let bytes = rkyv::to_bytes::<rkyv::rancor::Error>(&user_dict)
        .expect("Failed to serialize user dictionary");

    fs::write(&out_path, &bytes).expect("Failed to write user dictionary binary");

    println!(
        "cargo:warning=User dictionary built successfully ({} bytes)",
        bytes.len()
    );
}
