use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use safetensors::SafeTensors;
use safetensors::serialize;

fn main() {
    handle_lindera_dictionary();
    handle_yolo_model();
    println!("cargo:rerun-if-changed=build.rs");
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

fn handle_yolo_model() {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let yolo_dir = Path::new(&manifest_dir).join("public").join("yolo");
    let safetensors_path = yolo_dir.join("layout.safetensors");

    if safetensors_path.exists() {
        println!("cargo:warning=YOLO layout model already exists, skipping download");
        return;
    }

    println!("cargo:warning=Downloading and converting YOLO layout model...");

    fs::create_dir_all(&yolo_dir).expect("Failed to create yolo directory");

    // We download model.safetensors from juliozhao/DocLayout-YOLO-D4LA-from_scratch-HF
    // because it's already in safetensors format.
    let url = "https://huggingface.co/juliozhao/DocLayout-YOLO-D4LA-from_scratch-HF/resolve/main/model.safetensors";

    let response = reqwest::blocking::get(url).expect("Failed to download YOLO model");
    if !response.status().is_success() {
        panic!("Failed to download YOLO model: HTTP {}", response.status());
    }

    let bytes = response
        .bytes()
        .expect("Failed to get model bytes")
        .to_vec();

    // Rename keys to match our YoloV8 struct implementation
    let tensors = SafeTensors::deserialize(&bytes).expect("Failed to parse safetensors");
    let mut new_tensors = HashMap::new();

    for (name, tensor) in tensors.tensors() {
        if name.contains("anchors") || name.contains("strides") {
            continue;
        }

        let new_name = rename_yolo_key(&name);
        new_tensors.insert(new_name, tensor);
    }

    let out_data = serialize(new_tensors, &None).expect("Failed to serialize safetensors");
    fs::write(&safetensors_path, out_data).expect("Failed to write layout.safetensors");

    println!("cargo:warning=YOLO layout model downloaded and converted successfully");
}

fn rename_yolo_key(key: &str) -> String {
    let replacements = [
        ("model.0.", "net.b1.0."),
        ("model.1.", "net.b1.1."),
        ("model.2.m.", "net.b2.0.bottleneck."),
        ("model.2.", "net.b2.0."),
        ("model.3.", "net.b2.1."),
        ("model.4.m.", "net.b2.2.bottleneck."),
        ("model.4.", "net.b2.2."),
        ("model.5.", "net.b3.0."),
        ("model.6.m.", "net.b3.1.bottleneck."),
        ("model.6.", "net.b3.1."),
        ("model.7.", "net.b4.0."),
        ("model.8.m.", "net.b4.1.bottleneck."),
        ("model.8.", "net.b4.1."),
        ("model.9.", "net.b5.0."),
        ("model.12.m.", "fpn.n1.bottleneck."),
        ("model.12.", "fpn.n1."),
        ("model.15.m.", "fpn.n2.bottleneck."),
        ("model.15.", "fpn.n2."),
        ("model.16.", "fpn.n3."),
        ("model.18.m.", "fpn.n4.bottleneck."),
        ("model.18.", "fpn.n4."),
        ("model.19.", "fpn.n5."),
        ("model.21.m.", "fpn.n6.bottleneck."),
        ("model.21.", "fpn.n6."),
        ("model.22.", "head."),
    ];

    for (old, new) in replacements {
        if key.starts_with(old) {
            return key.replacen(old, new, 1);
        }
    }
    key.to_string()
}
