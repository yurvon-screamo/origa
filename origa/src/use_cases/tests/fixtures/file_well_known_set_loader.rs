use std::{io, path::PathBuf};

use super::get_public_dir;
use crate::domain::{JapaneseLevel, OrigaError};
use crate::traits::{WellKnownSet, WellKnownSetLoader, WellKnownSetMeta, resolve_set_path};

pub struct FileWellKnownSetLoader {
    public_dir: PathBuf,
}

impl FileWellKnownSetLoader {
    pub fn new() -> Self {
        Self {
            public_dir: get_public_dir(),
        }
    }
}

impl Default for FileWellKnownSetLoader {
    fn default() -> Self {
        Self::new()
    }
}

impl WellKnownSetLoader for FileWellKnownSetLoader {
    async fn load_meta_list(&self) -> Result<Vec<WellKnownSetMeta>, OrigaError> {
        let path = self
            .public_dir
            .join("domain")
            .join("well_known_set")
            .join("well_known_sets_meta.json");

        if !path.exists() {
            return Err(OrigaError::WellKnownSetNotFound {
                set_id: "well_known_sets_meta.json".to_string(),
            });
        }

        let json = std::fs::read_to_string(&path).map_err(|e| {
            if e.kind() == io::ErrorKind::NotFound {
                OrigaError::WellKnownSetNotFound {
                    set_id: "well_known_sets_meta.json".to_string(),
                }
            } else {
                OrigaError::WellKnownSetParseError {
                    reason: format!("Failed to read meta list: {}", e),
                }
            }
        })?;
        serde_json::from_str(&json).map_err(|e| OrigaError::WellKnownSetParseError {
            reason: format!("Failed to parse meta list: {}", e),
        })
    }

    async fn load_set(&self, id: String) -> Result<WellKnownSet, OrigaError> {
        #[derive(serde::Deserialize)]
        struct SetData {
            level: JapaneseLevel,
            words: Vec<String>,
        }

        let path = self.public_dir.join(resolve_set_path(&id));

        if !path.exists() {
            return Err(OrigaError::WellKnownSetNotFound { set_id: id.clone() });
        }

        let json = std::fs::read_to_string(&path).map_err(|e| {
            if e.kind() == io::ErrorKind::NotFound {
                OrigaError::WellKnownSetNotFound { set_id: id.clone() }
            } else {
                OrigaError::WellKnownSetParseError {
                    reason: format!("Failed to read set {}: {}", id, e),
                }
            }
        })?;

        let data: SetData =
            serde_json::from_str(&json).map_err(|e| OrigaError::WellKnownSetParseError {
                reason: format!("Failed to parse set {}: {}", id, e),
            })?;

        Ok(WellKnownSet::new(data.level, data.words))
    }
}
