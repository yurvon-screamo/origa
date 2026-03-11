use std::env;
use std::fs;
use std::path::Path;

fn main() {
    handle_lindera_dictionary();
    println!("cargo:rerun-if-changed=build.rs");

    // Version info from CI environment
    let version = option_env!("ORIGA_VERSION").unwrap_or("dev");
    let commit = option_env!("ORIGA_COMMIT").unwrap_or("unknown");
    let build_date = option_env!("ORIGA_BUILD_DATE").unwrap_or("unknown");

    println!("cargo:rustc-env=ORIGA_VERSION={}", version);
    println!("cargo:rustc-env=ORIGA_COMMIT={}", commit);
    println!("cargo:rustc-env=ORIGA_BUILD_DATE={}", build_date);

    println!("cargo:rerun-if-env-changed=ORIGA_VERSION");
    println!("cargo:rerun-if-env-changed=ORIGA_COMMIT");
    println!("cargo:rerun-if-env-changed=ORIGA_BUILD_DATE");
}

fn handle_lindera_dictionary() {
    if env::var("LINDERA_DICTIONARIES_PATH").is_ok() || env::var("LINDERA_CACHE").is_ok() {
        println!("cargo:warning=Using external dictionary path, skipping build");
        return;
    }

    let out_dir = env::var("OUT_DIR").unwrap();
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let public_dict_dir = Path::new(&manifest_dir)
        .join("public")
        .join("dictionaries")
        .join("unidic");

    if public_dict_dir.exists() {
        let dict_da = public_dict_dir.join("dict.da");
        if dict_da.exists() {
            println!("cargo:warning=Dictionary already exists, skipping build");
            return;
        }
    }

    println!("cargo:warning=Building UniDic dictionary...");

    fs::create_dir_all(&public_dict_dir).expect("Failed to create dictionary directory");

    let fetch_params = lindera_dictionary::assets::FetchParams {
        file_name: "unidic-mecab-2.1.2.tar.gz",
        input_dir: "unidic-mecab-2.1.2",
        output_dir: "lindera-unidic",
        dummy_input: "テスト,5131,5131,767,名詞,普通名詞,サ変可能,*,*,*,テスト,テスト-test,テスト,テスト,テスト,テスト,外,*,*,*,*\n",
        download_urls: &["https://Lindera.dev/unidic-mecab-2.1.2.tar.gz"],
        md5_hash: "f4502a563e1da44747f61dcd2b269e35",
    };

    let metadata_json = r#"{
        "name": "unidic",
        "encoding": "UTF-8",
        "compress_algorithm": "deflate",
        "default_word_cost": -10000,
        "default_left_context_id": 0,
        "default_right_context_id": 0,
        "default_field_value": "*",
        "flexible_csv": false,
        "skip_invalid_cost_or_id": false,
        "normalize_details": false,
        "dictionary_schema": {
            "fields": [
                "surface",
                "left_context_id",
                "right_context_id",
                "cost",
                "part_of_speech",
                "part_of_speech_subcategory_1",
                "part_of_speech_subcategory_2",
                "part_of_speech_subcategory_3",
                "conjugation_type",
                "conjugation_form",
                "reading",
                "lexeme",
                "orthographic_surface_form",
                "phonological_surface_form",
                "orthographic_base_form",
                "phonological_base_form",
                "word_type",
                "initial_mutation_type",
                "initial_mutation_form",
                "final_mutation_type",
                "final_mutation_form"
            ]
        },
        "user_dictionary_schema": {
            "fields": [
                "surface",
                "part_of_speech",
                "reading"
            ]
        }
    }"#;

    let metadata: lindera_dictionary::dictionary::metadata::Metadata =
        serde_json::from_str(metadata_json).expect("Failed to parse metadata");

    let builder = lindera_dictionary::builder::DictionaryBuilder::new(metadata);

    let runtime = tokio::runtime::Runtime::new().expect("Failed to create tokio runtime");
    runtime.block_on(async {
        lindera_dictionary::assets::fetch(fetch_params, builder)
            .await
            .expect("Failed to fetch and build dictionary");
    });

    let built_dict_dir = Path::new(&out_dir).join("lindera-unidic");

    let files_to_copy = [
        "char_def.bin",
        "matrix.mtx",
        "dict.da",
        "dict.vals",
        "unk.bin",
        "dict.wordsidx",
        "dict.words",
        "metadata.json",
    ];

    for file in &files_to_copy {
        let src = built_dict_dir.join(file);
        let dst = public_dict_dir.join(file);
        if src.exists() {
            fs::copy(&src, &dst).unwrap_or_else(|_| panic!("Failed to copy {}", file));
        } else {
            panic!("Dictionary file not found: {}", src.display());
        }
    }

    println!("cargo:warning=UniDic dictionary built successfully");
}
