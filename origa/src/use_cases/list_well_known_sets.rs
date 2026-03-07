use tracing::{debug, info};

use crate::traits::{WellKnownSetLoader, WellKnownSetMeta};
use crate::domain::OrigaError;

#[derive(Debug, Clone)]
pub struct WellKnownSetInfo {
    pub meta: WellKnownSetMeta,
    pub word_count: Option<usize>,
}

impl WellKnownSetInfo {
    pub fn new(meta: WellKnownSetMeta, word_count: Option<usize>) -> Self {
        Self { meta, word_count }
    }
}

pub struct ListWellKnownSetsUseCase<'a, L: WellKnownSetLoader> {
    loader: &'a L,
}

impl<'a, L: WellKnownSetLoader> ListWellKnownSetsUseCase<'a, L> {
    pub fn new(loader: &'a L) -> Self {
        Self { loader }
    }

    pub async fn execute(&self) -> Result<Vec<WellKnownSetInfo>, OrigaError> {
        debug!("Listing well-known sets");

        let meta_list = self.loader.load_meta_list().await?;

        let result: Vec<WellKnownSetInfo> = meta_list
            .into_iter()
            .map(|meta| WellKnownSetInfo::new(meta, None))
            .collect();

        info!(count = result.len(), "Well-known sets listed");
        Ok(result)
    }
}
