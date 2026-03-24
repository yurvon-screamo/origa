use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::path::Path;

fn main() {
    handle_lindera_dictionary();
    generate_well_known_meta();
    println!("cargo:rerun-if-changed=build.rs");

    // Version info from CI environment
    let version = option_env!("ORIGA_VERSION").unwrap_or("dev");
    let commit = option_env!("ORIGA_COMMIT").unwrap_or("unknown");
    let build_date = option_env!("ORIGA_BUILD_DATE").unwrap_or("unknown");
    let public_base_url = option_env!("PUBLIC_BASE_URL").unwrap_or("");

    println!("cargo:rustc-env=ORIGA_VERSION={}", version);
    println!("cargo:rustc-env=ORIGA_COMMIT={}", commit);
    println!("cargo:rustc-env=ORIGA_BUILD_DATE={}", build_date);
    println!("cargo:rustc-env=PUBLIC_BASE_URL={}", public_base_url);

    println!("cargo:rerun-if-env-changed=ORIGA_VERSION");
    println!("cargo:rerun-if-env-changed=ORIGA_COMMIT");
    println!("cargo:rerun-if-env-changed=ORIGA_BUILD_DATE");
    println!("cargo:rerun-if-env-changed=PUBLIC_BASE_URL");
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

#[derive(Debug, Default, Deserialize)]
#[allow(non_snake_case)]
struct WellKnownContent {
    #[serde(default)]
    Russian: Option<LanguageContent>,
    #[serde(default)]
    English: Option<LanguageContent>,
}

#[derive(Debug, Deserialize)]
struct LanguageContent {
    #[serde(default)]
    title: String,
    #[serde(default)]
    description: String,
}

#[derive(Debug, Deserialize)]
struct WellKnownSet {
    #[serde(default)]
    content: WellKnownContent,
    #[serde(default)]
    words: Vec<serde_json::Value>,
    #[serde(default)]
    level: String,
}

#[derive(Debug, Serialize)]
struct WellKnownMeta {
    id: String,
    set_type: String,
    level: String,
    title_ru: String,
    title_en: String,
    desc_ru: String,
    desc_en: String,
    word_count: usize,
}

fn extract_meta(
    data: &mut WellKnownSet,
    set_id: &str,
    set_type: &str,
    level: &str,
) -> WellKnownMeta {
    data.level = level.to_string();

    let (title_ru, desc_ru) = data
        .content
        .Russian
        .as_ref()
        .map(|r| (r.title.clone(), r.description.clone()))
        .unwrap_or_default();

    let (title_en, desc_en) = data
        .content
        .English
        .as_ref()
        .map(|e| (e.title.clone(), e.description.clone()))
        .unwrap_or_default();

    WellKnownMeta {
        id: set_id.to_string(),
        set_type: set_type.to_string(),
        level: level.to_string(),
        title_ru,
        title_en,
        desc_ru,
        desc_en,
        word_count: data.words.len(),
    }
}

fn load_json(path: &Path) -> WellKnownSet {
    let content =
        fs::read_to_string(path).unwrap_or_else(|_| panic!("Failed to read {}", path.display()));
    serde_json::from_str(&content)
        .unwrap_or_else(|_| panic!("Failed to parse JSON from {}", path.display()))
}

fn generate_well_known_meta() {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let base_dir = Path::new(&manifest_dir)
        .join("public")
        .join("domain")
        .join("well_known_set");
    let output_file = base_dir.join("well_known_sets_meta.json");

    // Rerun if any JSON file in well_known_set changes
    println!(
        "cargo:rerun-if-changed={}",
        base_dir.join("jlpt_n5.json").display()
    );
    println!(
        "cargo:rerun-if-changed={}",
        base_dir.join("jlpt_n4.json").display()
    );
    println!(
        "cargo:rerun-if-changed={}",
        base_dir.join("jlpt_n3.json").display()
    );
    println!(
        "cargo:rerun-if-changed={}",
        base_dir.join("jlpt_n2.json").display()
    );
    println!(
        "cargo:rerun-if-changed={}",
        base_dir.join("jlpt_n1.json").display()
    );
    println!(
        "cargo:rerun-if-changed={}",
        base_dir.join("migii").display()
    );
    println!(
        "cargo:rerun-if-changed={}",
        base_dir.join("minna_n5").display()
    );
    println!(
        "cargo:rerun-if-changed={}",
        base_dir.join("minna_n4").display()
    );
    println!(
        "cargo:rerun-if-changed={}",
        base_dir.join("spy_family").display()
    );
    println!(
        "cargo:rerun-if-changed={}",
        base_dir.join("duolingo").display()
    );

    let mut meta_list: Vec<WellKnownMeta> = Vec::new();

    // JLPT files
    let jlpt_files = [
        ("jlpt_n5.json", "jlpt_n5", "N5"),
        ("jlpt_n4.json", "jlpt_n4", "N4"),
        ("jlpt_n3.json", "jlpt_n3", "N3"),
        ("jlpt_n2.json", "jlpt_n2", "N2"),
        ("jlpt_n1.json", "jlpt_n1", "N1"),
    ];

    for (filename, set_id, level) in &jlpt_files {
        let path = base_dir.join(filename);
        if path.exists() {
            let mut data = load_json(&path);
            meta_list.push(extract_meta(&mut data, set_id, "Jlpt", level));
        }
    }

    // Migii files
    let migii_configs = [
        ("n5", 20, "N5"),
        ("n4", 11, "N4"),
        ("n3", 31, "N3"),
        ("n2", 31, "N2"),
        ("n1", 56, "N1"),
    ];

    for (level_id, count, level) in &migii_configs {
        for i in 1..=*count {
            let filename = format!("migii/{}/migii_{}_{}.json", level_id, level_id, i);
            let path = base_dir.join(&filename);
            if path.exists() {
                let mut data = load_json(&path);
                let set_id = format!("migii_{}_{}", level_id, i);
                meta_list.push(extract_meta(&mut data, &set_id, "Migii", level));
            }
        }
    }

    // Minna N5 files
    let minna_n5_dir = base_dir.join("minna_n5");
    if minna_n5_dir.exists() {
        let mut entries: Vec<_> = fs::read_dir(&minna_n5_dir)
            .unwrap()
            .filter_map(|e| e.ok())
            .filter(|e| {
                e.path()
                    .extension()
                    .map(|ext| ext == "json")
                    .unwrap_or(false)
            })
            .collect();
        entries.sort_by_key(|e| e.file_name());

        for entry in entries {
            let path = entry.path();
            if path
                .file_name()
                .map(|n| n.to_string_lossy().starts_with("minna_n5_"))
                .unwrap_or(false)
            {
                let mut data = load_json(&path);
                let set_id = path.file_stem().unwrap().to_string_lossy().to_string();
                meta_list.push(extract_meta(&mut data, &set_id, "MinnaNoNihongo", "N5"));
            }
        }
    }

    // Minna N4 files
    let minna_n4_dir = base_dir.join("minna_n4");
    if minna_n4_dir.exists() {
        let mut entries: Vec<_> = fs::read_dir(&minna_n4_dir)
            .unwrap()
            .filter_map(|e| e.ok())
            .filter(|e| {
                e.path()
                    .extension()
                    .map(|ext| ext == "json")
                    .unwrap_or(false)
            })
            .collect();
        entries.sort_by_key(|e| e.file_name());

        for entry in entries {
            let path = entry.path();
            if path
                .file_name()
                .map(|n| n.to_string_lossy().starts_with("minna_n4_"))
                .unwrap_or(false)
            {
                let mut data = load_json(&path);
                let set_id = path.file_stem().unwrap().to_string_lossy().to_string();
                meta_list.push(extract_meta(&mut data, &set_id, "MinnaNoNihongo", "N4"));
            }
        }
    }

    // Spy Family files
    let spy_family_dir = base_dir.join("spy_family");
    if spy_family_dir.exists() {
        let mut entries: Vec<_> = fs::read_dir(&spy_family_dir)
            .unwrap()
            .filter_map(|e| e.ok())
            .filter(|e| {
                e.path()
                    .extension()
                    .map(|ext| ext == "json")
                    .unwrap_or(false)
            })
            .collect();
        entries.sort_by_key(|e| e.file_name());

        for entry in entries {
            let path = entry.path();
            let mut data = load_json(&path);
            let set_id = path.file_stem().unwrap().to_string_lossy().to_string();
            meta_list.push(extract_meta(&mut data, &set_id, "SpyFamily", "N5"));
        }
    }

    // Duolingo files
    let duolingo_dir = base_dir.join("duolingo");
    if duolingo_dir.exists() {
        let mut entries: Vec<_> = fs::read_dir(&duolingo_dir)
            .unwrap()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().map(|t| t.is_dir()).unwrap_or(false))
            .collect();
        entries.sort_by_key(|e| e.file_name());

        for subdir_entry in entries {
            let subdir = subdir_entry.path();
            let mut json_entries: Vec<_> = fs::read_dir(&subdir)
                .unwrap()
                .filter_map(|e| e.ok())
                .filter(|e| {
                    e.path()
                        .extension()
                        .map(|ext| ext == "json")
                        .unwrap_or(false)
                })
                .collect();
            json_entries.sort_by_key(|e| e.file_name());

            for json_entry in json_entries {
                let path = json_entry.path();
                let stem = path.file_stem().unwrap().to_string_lossy();
                let parent_name = path
                    .parent()
                    .unwrap()
                    .file_name()
                    .unwrap()
                    .to_string_lossy();
                let set_id = format!("duolingo_{}_{}", parent_name, stem);
                let set_type = if stem.contains("_en_") {
                    "DuolingoEn"
                } else {
                    "DuolingoRu"
                };

                let mut data = load_json(&path);
                meta_list.push(extract_meta(&mut data, &set_id, set_type, "N5"));
            }
        }
    }

    // Sort by ID
    meta_list.sort_by(|a, b| a.id.cmp(&b.id));

    // Write output
    let output = serde_json::to_string_pretty(&meta_list).expect("Failed to serialize meta list");
    fs::write(&output_file, output).expect("Failed to write output file");

    println!(
        "cargo:warning=Generated {} meta entries to {}",
        meta_list.len(),
        output_file.display()
    );
}
