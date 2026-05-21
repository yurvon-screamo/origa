use image::DynamicImage;
use tracing::info;

use super::cascade_wasm::CascadeRecognizer;
use super::deim_wasm::DeimDetector;
use super::shared::{ModelFiles, crop_bbox};
use crate::domain::OrigaError;

pub struct JapaneseOCRModel {
    detector: DeimDetector,
    recognizer: CascadeRecognizer,
}

impl JapaneseOCRModel {
    pub async fn from_model_files(files: ModelFiles) -> Result<Self, OrigaError> {
        info!("Initializing Japanese OCR model (NDLOCR-Lite) for WASM");

        let detector = DeimDetector::new(&files.deim).await?;
        let recognizer = CascadeRecognizer::new(
            &files.parseq30,
            &files.parseq50,
            &files.parseq100,
            &files.vocab,
        )
        .await?;

        Ok(Self {
            detector,
            recognizer,
        })
    }

    pub async fn run(&self, img: &DynamicImage) -> Result<String, OrigaError> {
        info!("Running OCR (NDLOCR-Lite)");

        let mut boxes = self.detector.detect(img).await?;
        if boxes.is_empty() {
            info!("No text detected");
            return Ok(String::new());
        }

        super::reading_order::sort_reading_order(&mut boxes, img.height(), img.width());

        let mut results = Vec::with_capacity(boxes.len());
        for bbox in boxes {
            let line_img = crop_bbox(img, &bbox);
            let text = self
                .recognizer
                .recognize(&line_img, bbox.pred_char_cnt)
                .await;
            if !text.is_empty() {
                results.push(text);
            }
        }

        let result = results.join("\n");
        info!(result_length = result.len(), "OCR completed");
        Ok(result)
    }
}
