use origa::{
    domain::{JapaneseLevel, OrigaError},
    traits::{WellKnownSet, WellKnownSetLoader, WellKnownSetMeta},
};
use serde::Deserialize;
use std::sync::OnceLock;

static META_CACHE: OnceLock<Vec<WellKnownSetMeta>> = OnceLock::new();

fn parse_well_known_meta_list(json: &str) -> Result<Vec<WellKnownSetMeta>, OrigaError> {
    serde_json::from_str(json).map_err(|e| OrigaError::WellKnownSetParseError {
        reason: format!("Error parsing meta list: {}", e),
    })
}

fn parse_well_known_set(json: &str, id: &str) -> Result<WellKnownSet, OrigaError> {
    #[derive(Deserialize)]
    struct SetData {
        level: JapaneseLevel,
        words: Vec<String>,
    }

    let data: SetData =
        serde_json::from_str(json).map_err(|e| OrigaError::WellKnownSetParseError {
            reason: format!("Error parsing {}: {}", id, e),
        })?;

    Ok(WellKnownSet::new(data.level, data.words))
}

#[derive(Clone)]
pub struct WellKnownSetLoaderImpl;

impl WellKnownSetLoaderImpl {
    pub fn new() -> Self {
        Self
    }
}

impl Default for WellKnownSetLoaderImpl {
    fn default() -> Self {
        Self::new()
    }
}

impl WellKnownSetLoader for WellKnownSetLoaderImpl {
    async fn load_meta_list(&self) -> Result<Vec<WellKnownSetMeta>, OrigaError> {
        if let Some(cached) = META_CACHE.get() {
            return Ok(cached.clone());
        }
        let json = fetch_text("domain/well_known_set/well_known_sets_meta.json").await?;
        let meta_list = parse_well_known_meta_list(&json)?;
        let _ = META_CACHE.set(meta_list.clone());
        Ok(meta_list)
    }

    async fn load_set(&self, id: String) -> Result<WellKnownSet, OrigaError> {
        let meta_list = self.load_meta_list().await?;
        let _meta = meta_list.iter().find(|m| m.id == id).ok_or_else(|| {
            OrigaError::WellKnownSetParseError {
                reason: format!("Set not found: {}", id),
            }
        })?;
        let path = id_to_path(&id);
        let json = fetch_text(&path).await?;
        parse_well_known_set(&json, &id)
    }
}

async fn fetch_text(url: &str) -> Result<String, OrigaError> {
    use leptos::wasm_bindgen::JsCast;
    use wasm_bindgen_futures::JsFuture;

    let url = format!("/public/{}", url);

    let window = web_sys::window().ok_or_else(|| OrigaError::WellKnownSetParseError {
        reason: "No window found".to_string(),
    })?;

    let resp_value = JsFuture::from(window.fetch_with_str(&url))
        .await
        .map_err(|e| OrigaError::WellKnownSetParseError {
            reason: format!("Failed to fetch {}: {:?}", url, e),
        })?;

    let resp: web_sys::Response =
        resp_value
            .dyn_into()
            .map_err(|e| OrigaError::WellKnownSetParseError {
                reason: format!("Failed to cast response for {}: {:?}", url, e),
            })?;

    if !resp.ok() {
        return Err(OrigaError::WellKnownSetParseError {
            reason: format!("Failed to fetch {}: HTTP {}", url, resp.status()),
        });
    }

    let text = JsFuture::from(
        resp.text()
            .map_err(|e| OrigaError::WellKnownSetParseError {
                reason: format!("Failed to get text promise for {}: {:?}", url, e),
            })?,
    )
    .await
    .map_err(|e| OrigaError::WellKnownSetParseError {
        reason: format!("Failed to read text for {}: {:?}", url, e),
    })?;

    text.as_string()
        .ok_or_else(|| OrigaError::WellKnownSetParseError {
            reason: format!("Response is not a string for {}", url),
        })
}

fn id_to_path(id: &str) -> String {
    if let Some(level) = id.strip_prefix("jlpt_") {
        format!("domain/well_known_set/jltp_{}.json", level)
    } else if let Some(rest) = id.strip_prefix("migii_") {
        let level = rest.split('_').next().unwrap_or("");
        format!("domain/well_known_set/migii/{}/{}.json", level, id)
    } else if let Some(rest) = id.strip_prefix("duolingo_") {
        let level = rest.split('_').next().unwrap_or("");
        let filename = rest.split_once('_').map(|(_, f)| f).unwrap_or("");
        format!("domain/well_known_set/duolingo/{}/{}.json", level, filename)
    } else {
        format!("domain/well_known_set/{}.json", id)
    }
}
