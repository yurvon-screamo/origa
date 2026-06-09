use std::env;
use std::fs;
use std::path::Path;

fn main() {
    build_user_dictionary();
}

fn build_user_dictionary() {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let tokenizer_dir = Path::new(&manifest_dir)
        .join("src")
        .join("domain")
        .join("tokenizer");

    let metadata_path = tokenizer_dir.join("metadata.json");
    let csv_path = tokenizer_dir.join("user_dictionary.csv");

    let out_dir = env::var("OUT_DIR").unwrap();
    let out_path = Path::new(&out_dir).join("user_dictionary.bin");

    if !metadata_path.exists() || !csv_path.exists() {
        println!("cargo:warning=Skipping user dictionary build: metadata or CSV not found");
        fs::write(&out_path, &[] as &[u8])
            .expect("Failed to write empty user dictionary placeholder");
        return;
    }

    println!("cargo:rerun-if-changed=src/domain/tokenizer/user_dictionary.csv");
    println!("cargo:rerun-if-changed=src/domain/tokenizer/metadata.json");

    let metadata_bytes = fs::read(&metadata_path).expect("Failed to read metadata.json");
    let metadata: lindera_dictionary::dictionary::metadata::Metadata =
        serde_json::from_slice(&metadata_bytes).expect("Failed to parse metadata.json");

    let builder = lindera_dictionary::builder::DictionaryBuilder::new(metadata);
    let user_dict = builder
        .build_user_dict(&csv_path)
        .expect("Failed to build user dictionary from CSV");

    let bytes = rkyv::to_bytes::<rkyv::rancor::Error>(&user_dict)
        .expect("Failed to serialize user dictionary");

    fs::write(&out_path, &bytes).expect("Failed to write user dictionary binary");

    println!(
        "cargo:warning=User dictionary built successfully ({} bytes)",
        bytes.len()
    );
}
