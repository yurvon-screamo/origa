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

    #[cfg(not(target_arch = "wasm32"))]
    pub fn execute(
        &self,
        model: &JapaneseOCRModel,
        image_bytes: &[u8],
    ) -> Result<String, OrigaError> {
        info!(
            bytes_len = image_bytes.len(),
            "Executing ExtractTextFromImageUseCase"
        );

        let img = decode_image(image_bytes)?;
        model.run(&img)
    }

    #[cfg(target_arch = "wasm32")]
    pub async fn execute(
        &self,
        model: &JapaneseOCRModel,
        image_bytes: &[u8],
    ) -> Result<String, OrigaError> {
        info!(
            bytes_len = image_bytes.len(),
            "Executing ExtractTextFromImageUseCase"
        );

        let img = decode_image(image_bytes)?;
        model.run(&img).await
    }
}

fn decode_image(image_bytes: &[u8]) -> Result<image::DynamicImage, OrigaError> {
    image::load_from_memory(image_bytes).map_err(|e| OrigaError::OcrError {
        reason: format!("Failed to decode image: {}", e),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_creates_instance() {
        let _use_case = ExtractTextFromImageUseCase::new();
    }

    #[test]
    fn default_creates_instance() {
        let _use_case = ExtractTextFromImageUseCase::new();
    }

    mod decode_image {
        use super::*;

        #[test]
        fn empty_bytes_returns_error() {
            let result = decode_image(&[]);
            assert!(matches!(result, Err(OrigaError::OcrError { .. })));
        }

        #[test]
        fn invalid_bytes_returns_error() {
            let invalid_bytes: &[u8] = &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9];
            let result = decode_image(invalid_bytes);
            assert!(matches!(result, Err(OrigaError::OcrError { .. })));
        }

        #[test]
        fn random_bytes_returns_error() {
            let random_bytes: Vec<u8> = (0..100).collect();
            let result = decode_image(&random_bytes);
            assert!(matches!(result, Err(OrigaError::OcrError { .. })));
        }

        #[test]
        fn truncated_png_header_returns_error() {
            let truncated_png: &[u8] = &[0x89, 0x50, 0x4E, 0x47];
            let result = decode_image(truncated_png);
            assert!(matches!(result, Err(OrigaError::OcrError { .. })));
        }

        #[test]
        fn truncated_jpeg_header_returns_error() {
            let truncated_jpeg: &[u8] = &[0xFF, 0xD8, 0xFF];
            let result = decode_image(truncated_jpeg);
            assert!(matches!(result, Err(OrigaError::OcrError { .. })));
        }

        #[test]
        fn error_message_contains_decode_failure() {
            let result = decode_image(&[0, 1, 2, 3]);
            if let Err(OrigaError::OcrError { reason }) = result {
                assert!(reason.contains("Failed to decode image"));
            } else {
                panic!("Expected OcrError");
            }
        }

        #[test]
        fn valid_png_image_decodes_successfully() {
            let png_bytes = create_minimal_png();
            let result = decode_image(&png_bytes);
            assert!(result.is_ok());
        }

        #[test]
        fn valid_jpeg_image_decodes_successfully() {
            let jpeg_bytes = create_minimal_jpeg();
            let result = decode_image(&jpeg_bytes);
            assert!(result.is_ok());
        }

        fn create_minimal_png() -> Vec<u8> {
            let img = image::DynamicImage::ImageRgba8(image::RgbaImage::new(1, 1));
            let mut bytes = Vec::new();
            img.write_to(
                &mut std::io::Cursor::new(&mut bytes),
                image::ImageFormat::Png,
            )
            .unwrap();
            bytes
        }

        fn create_minimal_jpeg() -> Vec<u8> {
            let img = image::DynamicImage::ImageRgb8(image::RgbImage::new(1, 1));
            let mut bytes = Vec::new();
            img.write_to(
                &mut std::io::Cursor::new(&mut bytes),
                image::ImageFormat::Jpeg,
            )
            .unwrap();
            bytes
        }
    }
}
