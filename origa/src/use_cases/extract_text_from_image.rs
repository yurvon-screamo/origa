use crate::domain::OrigaError;
use crate::ocr::JapaneseOCRModel;
use tracing::info;

pub struct ExtractTextFromImageUseCase;

impl Default for ExtractTextFromImageUseCase {
    fn default() -> Self {
        Self::new()
    }
}

impl ExtractTextFromImageUseCase {
    pub fn new() -> Self {
        Self
    }

    pub fn execute(
        &self,
        model: &mut JapaneseOCRModel,
        image_bytes: &[u8],
    ) -> Result<String, OrigaError> {
        info!(
            bytes_len = image_bytes.len(),
            "Executing ExtractTextFromImageUseCase"
        );

        let img = image::load_from_memory(image_bytes).map_err(|e| OrigaError::OcrError {
            reason: format!("Failed to decode image: {}", e),
        })?;

        model.run(&img)
    }
}
