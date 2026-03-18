use futures::future::join_all;
use origa::{
    domain::{JapaneseLevel, OrigaError},
    traits::{WellKnownSet, WellKnownSetLoader, WellKnownSetMeta},
};
use serde::Deserialize;
use std::sync::OnceLock;

use crate::core::config::public_url;
use crate::utils::fetch_text;

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
        let json = fetch_text(public_url(
            "/public/domain/well_known_set/well_known_sets_meta.json",
        ))
        .await?;
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
        let json = fetch_text(public_url(&format!("/public/{}", path))).await?;
        parse_well_known_set(&json, &id)
    }

    async fn load_sets(&self, ids: Vec<String>) -> Result<Vec<(String, WellKnownSet)>, OrigaError> {
        let futures: Vec<_> = ids
            .into_iter()
            .map(|id| {
                let id_clone = id.clone();
                async move {
                    let set = self.load_set(id_clone.clone()).await?;
                    Ok::<_, OrigaError>((id_clone, set))
                }
            })
            .collect();

        let results = join_all(futures)
            .await
            .into_iter()
            .collect::<Result<Vec<_>, _>>()?;

        Ok(results)
    }
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
