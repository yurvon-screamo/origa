use image::DynamicImage;
use tracing::info;

use super::cascade_wasm::CascadeRecognizer;
use super::deim_wasm::DeimDetector;
use crate::domain::OrigaError;

pub struct ModelFiles {
    pub deim: Vec<u8>,
    pub parseq30: Vec<u8>,
    pub parseq50: Vec<u8>,
    pub parseq100: Vec<u8>,
    pub vocab: Vec<u8>,
}

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

fn crop_bbox(image: &DynamicImage, bbox: &super::types::BoundingBox) -> DynamicImage {
    let x0 = bbox.x0.max(0) as u32;
    let y0 = bbox.y0.max(0) as u32;
    let x1 = bbox.x1.max(0) as u32;
    let y1 = bbox.y1.max(0) as u32;

    if x1 <= x0 || y1 <= y0 {
        return image.crop_imm(0, 0, 1, 1);
    }

    let x0 = x0.min(image.width());
    let y0 = y0.min(image.height());
    let x1 = x1.min(image.width());
    let y1 = y1.min(image.height());

    let width = x1 - x0;
    let height = y1 - y0;

    if width == 0 || height == 0 {
        return image.crop_imm(0, 0, 1, 1);
    }

    image.crop_imm(x0, y0, width, height)
}
