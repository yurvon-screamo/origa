use futures::future::join_all;
use origa::domain::{
    JapaneseLevel, OrigaError, TypesMeta, WellKnownSet, WellKnownSetMeta, get_types_meta,
    resolve_set_path, set_types_meta,
};
use origa::traits::CdnProvider;
use serde::Deserialize;
use std::sync::OnceLock;

use crate::repository::cdn_provider;

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

    async fn ensure_types_loaded() -> Result<(), OrigaError> {
        if get_types_meta().is_some() {
            return Ok(());
        }
        let cdn = cdn_provider();
        let json = cdn
            .fetch_text("well_known_set/well_known_types_meta.json")
            .await?;
        let types_meta: TypesMeta =
            serde_json::from_str(&json).map_err(|e| OrigaError::WellKnownSetParseError {
                reason: format!("Error parsing types meta: {}", e),
            })?;
        set_types_meta(types_meta);
        Ok(())
    }

    pub async fn load_meta_list(&self) -> Result<Vec<WellKnownSetMeta>, OrigaError> {
        Self::ensure_types_loaded().await?;

        if let Some(cached) = META_CACHE.get() {
            return Ok(cached.clone());
        }
        let cdn = cdn_provider();
        let json = cdn
            .fetch_text("well_known_set/well_known_sets_meta.json")
            .await?;
        let meta_list = parse_well_known_meta_list(&json)?;
        let _ = META_CACHE.set(meta_list.clone());
        Ok(meta_list)
    }

    pub async fn load_set(&self, id: String) -> Result<WellKnownSet, OrigaError> {
        let meta_list = self.load_meta_list().await?;
        let _meta = meta_list.iter().find(|m| m.id == id).ok_or_else(|| {
            OrigaError::WellKnownSetParseError {
                reason: format!("Set not found: {}", id),
            }
        })?;
        let path = resolve_set_path(&id);
        let cdn = cdn_provider();
        let json = cdn.fetch_text(&path).await?;
        parse_well_known_set(&json, &id)
    }

    pub async fn load_sets(
        &self,
        ids: Vec<String>,
    ) -> Result<Vec<(String, WellKnownSet)>, OrigaError> {
        let futures: Vec<_> = ids
            .into_iter()
            .map(|id| {
                let id_clone = id.clone();
                async move {
                    let set = Self::load_set_owned(id_clone.clone()).await?;
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

    async fn load_set_owned(id: String) -> Result<WellKnownSet, OrigaError> {
        let loader = Self::new();
        loader.load_set(id).await
    }
}

impl Default for WellKnownSetLoaderImpl {
    fn default() -> Self {
        Self::new()
    }
}
