use origa::domain::{JftCandoCatalog, OrigaError};
use origa::traits::CdnProvider;

use crate::repository::cdn_provider;

const CANDO_PATH: &str = "well_known_set/irodori_cando.json";

/// Session cache for the parsed catalog (mirrors WellKnownSetLoaderImpl's
/// META_CACHE). Only successful loads are cached, so a transient offline
/// start retries on the next home mount instead of staying hidden forever.
static CATALOG_CACHE: std::sync::OnceLock<JftCandoCatalog> = std::sync::OnceLock::new();

#[derive(Clone)]
pub struct JftCandoLoaderImpl;

impl JftCandoLoaderImpl {
    pub fn new() -> Self {
        Self
    }

    /// Fetches and parses the Irodori → Can-do mapping. Failures are
    /// expected offline paths — the caller hides the readiness card.
    pub async fn load(&self) -> Result<JftCandoCatalog, OrigaError> {
        if let Some(catalog) = CATALOG_CACHE.get() {
            return Ok(catalog.clone());
        }
        let cdn = cdn_provider();
        let json = cdn.fetch_text(CANDO_PATH).await?;
        let catalog = JftCandoCatalog::parse(&json)?;
        let _ = CATALOG_CACHE.set(catalog.clone());
        Ok(catalog)
    }
}

impl Default for JftCandoLoaderImpl {
    fn default() -> Self {
        Self::new()
    }
}
