mod config;
mod reading_order;
mod types;
mod vocab;

#[cfg(not(target_arch = "wasm32"))]
mod cascade;
#[cfg(not(target_arch = "wasm32"))]
mod deim;
#[cfg(not(target_arch = "wasm32"))]
mod model;
#[cfg(not(target_arch = "wasm32"))]
mod parseq;

#[cfg(target_arch = "wasm32")]
mod cascade_wasm;
#[cfg(target_arch = "wasm32")]
mod deim_wasm;
#[cfg(target_arch = "wasm32")]
mod model_wasm;
#[cfg(target_arch = "wasm32")]
mod parseq_wasm;

#[cfg(test)]
mod tests;

pub use config::ModelConfig;
pub use reading_order::sort_reading_order;
pub use types::BoundingBox;
pub use vocab::Vocabulary;

#[cfg(not(target_arch = "wasm32"))]
pub use cascade::CascadeRecognizer;
#[cfg(not(target_arch = "wasm32"))]
pub use deim::DeimDetector;
#[cfg(not(target_arch = "wasm32"))]
pub use model::{JapaneseOCRModel, ModelFiles};
#[cfg(not(target_arch = "wasm32"))]
pub use parseq::ParseqRecognizer;

#[cfg(target_arch = "wasm32")]
pub use cascade_wasm::CascadeRecognizer;
#[cfg(target_arch = "wasm32")]
pub use deim_wasm::DeimDetector;
#[cfg(target_arch = "wasm32")]
pub use model_wasm::{JapaneseOCRModel, ModelFiles};
#[cfg(target_arch = "wasm32")]
pub use parseq_wasm::ParseqRecognizer;
