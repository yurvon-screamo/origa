use origa::domain::{JftCandoCatalog, OrigaError};
use origa::traits::CdnProvider;

use crate::repository::cdn_provider;

const CANDO_PATH: &str = "well_known_set/irodori_cando.json";

#[derive(Clone)]
pub struct JftCandoLoaderImpl;

impl JftCandoLoaderImpl {
    pub fn new() -> Self {
        Self
    }

    /// Fetches and parses the Irodori → Can-do mapping. Failures are
    /// expected offline paths — the caller hides the readiness card.
    pub async fn load(&self) -> Result<JftCandoCatalog, OrigaError> {
        let cdn = cdn_provider();
        let json = cdn.fetch_text(CANDO_PATH).await?;
        JftCandoCatalog::parse(&json)
    }
}

impl Default for JftCandoLoaderImpl {
    fn default() -> Self {
        Self::new()
    }
}
