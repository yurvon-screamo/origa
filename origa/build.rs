use std::{env, fs, path::Path};

fn main() {
    build_user_dictionary();
}

fn user_dictionary_row_handler(
    row: &csv::StringRecord,
) -> lindera_dictionary::LinderaResult<Vec<String>> {
    let part_of_speech = row.get(1).unwrap_or("*").to_string();
    let reading = row.get(2).unwrap_or("*").to_string();
    Ok(vec![
        part_of_speech,  // part_of_speech
        "*".to_string(), // part_of_speech_subcategory_1
        "*".to_string(), // part_of_speech_subcategory_2
        "*".to_string(), // part_of_speech_subcategory_3
        "*".to_string(), // conjugation_type
        "*".to_string(), // conjugation_form
        reading,         // reading
        "*".to_string(), // lexeme
        "*".to_string(), // orthographic_surface_form
        "*".to_string(), // phonological_surface_form
        "*".to_string(), // orthographic_base_form
        "*".to_string(), // phonological_base_form
        "*".to_string(), // word_type
        "*".to_string(), // initial_mutation_type
        "*".to_string(), // initial_mutation_form
        "*".to_string(), // final_mutation_type
        "*".to_string(), // final_mutation_form
    ])
}

fn build_user_dictionary() {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let csv_path = Path::new(&manifest_dir)
        .join("src")
        .join("domain")
        .join("tokenizer")
        .join("user_dictionary.csv");

    let out_dir = env::var("OUT_DIR").unwrap();
    let out_path = Path::new(&out_dir).join("user_dictionary.bin");

    if !csv_path.exists() {
        println!("cargo:warning=Skipping user dictionary build: CSV not found");
        fs::write(&out_path, &[] as &[u8])
            .expect("Failed to write empty user dictionary placeholder");
        return;
    }

    println!("cargo:rerun-if-changed=src/domain/tokenizer/user_dictionary.csv");

    let user_dict =
        lindera_dictionary::builder::user_dictionary::UserDictionaryBuilderOptions::default()
            .user_dictionary_fields_num(3)
            .dictionary_fields_num(21)
            .default_word_cost(-10000)
            .default_left_context_id(0)
            .default_right_context_id(0)
            .flexible_csv(false)
            .user_dictionary_handler(Some(Box::new(user_dictionary_row_handler)))
            .builder()
            .expect("Failed to build UserDictionaryBuilder")
            .build(&csv_path)
            .expect("Failed to build user dictionary from CSV");

    let bytes = rkyv::to_bytes::<rkyv::rancor::Error>(&user_dict)
        .expect("Failed to serialize user dictionary");

    fs::write(&out_path, &bytes).expect("Failed to write user dictionary binary");

    println!(
        "cargo:warning=User dictionary built successfully ({} bytes)",
        bytes.len()
    );
}
