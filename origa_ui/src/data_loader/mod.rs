use origa::domain::{
    GrammarData, KanjiData, OrigaError, RadicalData, VocabularyChunkData, WellKnownSetData,
    init_grammar_rules, init_kanji_dictionary, init_radical_dictionary, init_vocabulary_dictionary,
    init_well_known_sets, is_grammar_loaded, is_kanji_loaded, is_radical_loaded,
    is_vocabulary_loaded, is_well_known_sets_loaded,
};

pub fn is_all_data_loaded() -> bool {
    is_vocabulary_loaded()
        && is_radical_loaded()
        && is_kanji_loaded()
        && is_grammar_loaded()
        && is_well_known_sets_loaded()
}

#[cfg(target_arch = "wasm32")]
pub async fn load_all_data() -> Result<(), OrigaError> {
    if is_all_data_loaded() {
        return Ok(());
    }

    let (vocab_result, radical_result) = futures::join!(load_vocabulary(), load_radical());
    vocab_result?;
    radical_result?;

    let (kanji_result, grammar_result, wks_result) =
        futures::join!(load_kanji(), load_grammar(), load_well_known_sets());
    kanji_result?;
    grammar_result?;
    wks_result?;

    log::info!("All data loaded successfully");
    Ok(())
}

#[cfg(target_arch = "wasm32")]
async fn fetch_text(url: impl Into<String>) -> Result<String, OrigaError> {
    use leptos::wasm_bindgen::JsCast;
    use wasm_bindgen_futures::JsFuture;

    let url = format!("/public/{}", url.into());

    let window = web_sys::window().ok_or_else(|| OrigaError::TokenizerError {
        reason: "No window found".to_string(),
    })?;

    let resp_value = JsFuture::from(window.fetch_with_str(&url))
        .await
        .map_err(|e| OrigaError::TokenizerError {
            reason: format!("Failed to fetch {}: {:?}", url, e),
        })?;

    let resp: web_sys::Response =
        resp_value
            .dyn_into()
            .map_err(|e| OrigaError::TokenizerError {
                reason: format!("Failed to cast response for {}: {:?}", url, e),
            })?;

    if !resp.ok() {
        return Err(OrigaError::TokenizerError {
            reason: format!("Failed to fetch {}: HTTP {}", url, resp.status()),
        });
    }

    let text = JsFuture::from(resp.text().map_err(|e| OrigaError::TokenizerError {
        reason: format!("Failed to get text promise for {}: {:?}", url, e),
    })?)
    .await
    .map_err(|e| OrigaError::TokenizerError {
        reason: format!("Failed to read text for {}: {:?}", url, e),
    })?;

    text.as_string().ok_or_else(|| OrigaError::TokenizerError {
        reason: format!("Response is not a string for {}", url),
    })
}

#[cfg(target_arch = "wasm32")]
pub async fn load_vocabulary() -> Result<(), OrigaError> {
    if is_vocabulary_loaded() {
        return Ok(());
    }

    let chunk_futures: Vec<_> = (1..=10)
        .map(|i| fetch_text(format!("domain/dictionary/vocabulary/chunk_{:02}.json", i)))
        .collect();

    let chunks = join_all(chunk_futures).await;
    let chunks: Vec<String> = chunks.into_iter().collect::<Result<Vec<_>, _>>()?;

    let data = VocabularyChunkData {
        chunk_01: chunks[0].clone(),
        chunk_02: chunks[1].clone(),
        chunk_03: chunks[2].clone(),
        chunk_04: chunks[3].clone(),
        chunk_05: chunks[4].clone(),
        chunk_06: chunks[5].clone(),
        chunk_07: chunks[6].clone(),
        chunk_08: chunks[7].clone(),
        chunk_09: chunks[8].clone(),
        chunk_10: chunks[9].clone(),
    };

    init_vocabulary_dictionary(data)?;
    log::info!("Vocabulary loaded");
    Ok(())
}

#[cfg(target_arch = "wasm32")]
pub async fn load_radical() -> Result<(), OrigaError> {
    if is_radical_loaded() {
        return Ok(());
    }

    let json = fetch_text("domain/dictionary/radicals.json").await?;
    init_radical_dictionary(RadicalData {
        radicals_json: json,
    })?;
    log::info!("Radicals loaded");
    Ok(())
}

#[cfg(target_arch = "wasm32")]
pub async fn load_kanji() -> Result<(), OrigaError> {
    if is_kanji_loaded() {
        return Ok(());
    }

    let json = fetch_text("domain/dictionary/kanji.json").await?;
    init_kanji_dictionary(KanjiData { kanji_json: json })?;
    log::info!("Kanji loaded");
    Ok(())
}

#[cfg(target_arch = "wasm32")]
pub async fn load_grammar() -> Result<(), OrigaError> {
    if is_grammar_loaded() {
        return Ok(());
    }

    let json = fetch_text("domain/grammar/grammar.json").await?;
    init_grammar_rules(GrammarData { grammar_json: json })?;
    log::info!("Grammar loaded");
    Ok(())
}

#[cfg(target_arch = "wasm32")]
pub async fn load_well_known_sets() -> Result<(), OrigaError> {
    if is_well_known_sets_loaded() {
        return Ok(());
    }

    let mut fetch_futures: Vec<_> = Vec::new();

    fetch_futures.push(fetch_text("domain/well_known_set/jltp_n1.json".to_string()));
    fetch_futures.push(fetch_text("domain/well_known_set/jltp_n2.json".to_string()));
    fetch_futures.push(fetch_text("domain/well_known_set/jltp_n3.json".to_string()));
    fetch_futures.push(fetch_text("domain/well_known_set/jltp_n4.json".to_string()));
    fetch_futures.push(fetch_text("domain/well_known_set/jltp_n5.json".to_string()));

    for i in 1..=20 {
        fetch_futures.push(fetch_text(format!(
            "domain/well_known_set/migii/n5/migii_n5_{}.json",
            i
        )));
    }
    for i in 1..=11 {
        fetch_futures.push(fetch_text(format!(
            "domain/well_known_set/migii/n4/migii_n4_{}.json",
            i
        )));
    }
    for i in 1..=31 {
        fetch_futures.push(fetch_text(format!(
            "domain/well_known_set/migii/n3/migii_n3_{}.json",
            i
        )));
    }
    for i in 1..=31 {
        fetch_futures.push(fetch_text(format!(
            "domain/well_known_set/migii/n2/migii_n2_{}.json",
            i
        )));
    }
    for i in 1..=56 {
        fetch_futures.push(fetch_text(format!(
            "domain/well_known_set/migii/n1/migii_n1_{}.json",
            i
        )));
    }

    let results = join_all(fetch_futures).await;
    let all_jsons: Vec<String> = results.into_iter().collect::<Result<Vec<_>, _>>()?;

    let data = WellKnownSetData {
        jlpt_n1: all_jsons[0].clone(),
        jlpt_n2: all_jsons[1].clone(),
        jlpt_n3: all_jsons[2].clone(),
        jlpt_n4: all_jsons[3].clone(),
        jlpt_n5: all_jsons[4].clone(),
        migii_n5: all_jsons[5..25].to_vec(),
        migii_n4: all_jsons[25..36].to_vec(),
        migii_n3: all_jsons[36..67].to_vec(),
        migii_n2: all_jsons[67..98].to_vec(),
        migii_n1: all_jsons[98..154].to_vec(),
    };

    init_well_known_sets(data)?;
    log::info!("Well-known sets loaded");
    Ok(())
}

#[cfg(not(target_arch = "wasm32"))]
pub fn load_all_data() -> Result<(), OrigaError> {
    if is_all_data_loaded() {
        return Ok(());
    }

    load_vocabulary()?;
    load_radical()?;
    load_kanji()?;
    load_grammar()?;
    load_well_known_sets()?;

    log::info!("All data loaded successfully");
    Ok(())
}

#[cfg(not(target_arch = "wasm32"))]
fn read_json_file(path: &str) -> Result<String, OrigaError> {
    use std::{env, fs, path::PathBuf};

    let manifest_dir = env::var("CARGO_MANIFEST_DIR").map_err(|_| OrigaError::TokenizerError {
        reason: "CARGO_MANIFEST_DIR not set".to_string(),
    })?;

    let full_path = PathBuf::from(manifest_dir).join("public").join(path);

    fs::read_to_string(&full_path).map_err(|e| OrigaError::TokenizerError {
        reason: format!("Failed to read {}: {}", full_path.display(), e),
    })
}

#[cfg(not(target_arch = "wasm32"))]
pub fn load_vocabulary() -> Result<(), OrigaError> {
    if is_vocabulary_loaded() {
        return Ok(());
    }

    let mut chunks = Vec::with_capacity(10);
    for i in 1..=10 {
        let path = format!("domain/dictionary/vocabulary/chunk_{:02}.json", i);
        let json = read_json_file(&path)?;
        chunks.push(json);
    }

    let data = VocabularyChunkData {
        chunk_01: chunks[0].clone(),
        chunk_02: chunks[1].clone(),
        chunk_03: chunks[2].clone(),
        chunk_04: chunks[3].clone(),
        chunk_05: chunks[4].clone(),
        chunk_06: chunks[5].clone(),
        chunk_07: chunks[6].clone(),
        chunk_08: chunks[7].clone(),
        chunk_09: chunks[8].clone(),
        chunk_10: chunks[9].clone(),
    };

    init_vocabulary_dictionary(data)?;
    log::info!("Vocabulary loaded");
    Ok(())
}

#[cfg(not(target_arch = "wasm32"))]
pub fn load_radical() -> Result<(), OrigaError> {
    if is_radical_loaded() {
        return Ok(());
    }

    let json = read_json_file("domain/dictionary/radicals.json")?;
    init_radical_dictionary(RadicalData {
        radicals_json: json,
    })?;
    log::info!("Radicals loaded");
    Ok(())
}

#[cfg(not(target_arch = "wasm32"))]
pub fn load_kanji() -> Result<(), OrigaError> {
    if is_kanji_loaded() {
        return Ok(());
    }

    let json = read_json_file("domain/dictionary/kanji.json")?;
    init_kanji_dictionary(KanjiData { kanji_json: json })?;
    log::info!("Kanji loaded");
    Ok(())
}

#[cfg(not(target_arch = "wasm32"))]
pub fn load_grammar() -> Result<(), OrigaError> {
    if is_grammar_loaded() {
        return Ok(());
    }

    let json = read_json_file("domain/grammar/grammar.json")?;
    init_grammar_rules(GrammarData { grammar_json: json })?;
    log::info!("Grammar loaded");
    Ok(())
}

#[cfg(not(target_arch = "wasm32"))]
pub fn load_well_known_sets() -> Result<(), OrigaError> {
    if is_well_known_sets_loaded() {
        return Ok(());
    }

    let jlpt_n1 = read_json_file("domain/well_known_set/jltp_n1.json")?;
    let jlpt_n2 = read_json_file("domain/well_known_set/jltp_n2.json")?;
    let jlpt_n3 = read_json_file("domain/well_known_set/jltp_n3.json")?;
    let jlpt_n4 = read_json_file("domain/well_known_set/jltp_n4.json")?;
    let jlpt_n5 = read_json_file("domain/well_known_set/jltp_n5.json")?;

    let mut migii_n5 = Vec::new();
    for i in 1..=20 {
        let path = format!("domain/well_known_set/migii/n5/migii_n5_{}.json", i);
        migii_n5.push(read_json_file(&path)?);
    }

    let mut migii_n4 = Vec::new();
    for i in 1..=11 {
        let path = format!("domain/well_known_set/migii/n4/migii_n4_{}.json", i);
        migii_n4.push(read_json_file(&path)?);
    }

    let mut migii_n3 = Vec::new();
    for i in 1..=31 {
        let path = format!("domain/well_known_set/migii/n3/migii_n3_{}.json", i);
        migii_n3.push(read_json_file(&path)?);
    }

    let mut migii_n2 = Vec::new();
    for i in 1..=31 {
        let path = format!("domain/well_known_set/migii/n2/migii_n2_{}.json", i);
        migii_n2.push(read_json_file(&path)?);
    }

    let mut migii_n1 = Vec::new();
    for i in 1..=56 {
        let path = format!("domain/well_known_set/migii/n1/migii_n1_{}.json", i);
        migii_n1.push(read_json_file(&path)?);
    }

    let data = WellKnownSetData {
        jlpt_n1,
        jlpt_n2,
        jlpt_n3,
        jlpt_n4,
        jlpt_n5,
        migii_n5,
        migii_n4,
        migii_n3,
        migii_n2,
        migii_n1,
    };

    init_well_known_sets(data)?;
    log::info!("Well-known sets loaded");
    Ok(())
}
