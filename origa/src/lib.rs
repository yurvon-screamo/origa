pub mod dictionary;
pub mod domain;
pub mod ocr;
#[cfg(not(target_arch = "wasm32"))]
pub mod stt;
pub mod traits;
pub mod use_cases;
