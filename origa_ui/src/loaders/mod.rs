pub mod data_loader;
pub mod dictionary;
pub mod jlpt_content_loader;
pub mod ocr_model_loader;
pub mod well_known_set_loader;
#[cfg(target_arch = "wasm32")]
pub mod whisper_model_loader;

pub use jlpt_content_loader::{get_jlpt_content, recalculate_user_jlpt_progress};
pub use ocr_model_loader::ModelLoader;
pub use well_known_set_loader::WellKnownSetLoaderImpl;
#[cfg(target_arch = "wasm32")]
pub use whisper_model_loader::WhisperModelLoader;
