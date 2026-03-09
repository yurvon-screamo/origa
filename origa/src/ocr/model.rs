use std::path::PathBuf;

use image::DynamicImage;
use tempfile::tempdir;
use tracing::info;

use super::cascade::CascadeRecognizer;
use super::deim::DeimDetector;
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
    pub fn from_bytes(
        deim_bytes: Vec<u8>,
        parseq30_bytes: Vec<u8>,
        parseq50_bytes: Vec<u8>,
        parseq100_bytes: Vec<u8>,
        vocab_bytes: Vec<u8>,
    ) -> Result<Self, OrigaError> {
        info!("Initializing Japanese OCR model (NDLOCR-Lite)");

        let temp_dir = tempdir().map_err(|e| OrigaError::OcrError {
            reason: format!("Failed to create temp dir: {}", e),
        })?;

        let write_file = |name: &str, bytes: &[u8]| -> Result<PathBuf, OrigaError> {
            let path = temp_dir.path().join(name);
            std::fs::write(&path, bytes).map_err(|e| OrigaError::OcrError {
                reason: format!("Failed to write {}: {}", name, e),
            })?;
            Ok(path)
        };

        let deim_path = write_file("deim.onnx", &deim_bytes)?;
        let parseq30_path = write_file("parseq30.onnx", &parseq30_bytes)?;
        let parseq50_path = write_file("parseq50.onnx", &parseq50_bytes)?;
        let parseq100_path = write_file("parseq100.onnx", &parseq100_bytes)?;
        let vocab_path = write_file("vocab.txt", &vocab_bytes)?;

        let detector = DeimDetector::new(&deim_path)?;
        let recognizer =
            CascadeRecognizer::new(&parseq30_path, &parseq50_path, &parseq100_path, &vocab_path)?;

        std::mem::forget(temp_dir);

        Ok(Self {
            detector,
            recognizer,
        })
    }

    pub fn from_model_files(files: ModelFiles) -> Result<Self, OrigaError> {
        Self::from_bytes(
            files.deim,
            files.parseq30,
            files.parseq50,
            files.parseq100,
            files.vocab,
        )
    }

    pub fn run(&mut self, img: &DynamicImage) -> Result<String, OrigaError> {
        info!("Running OCR (NDLOCR-Lite)");

        let mut boxes = self.detector.detect(img)?;
        if boxes.is_empty() {
            info!("No text detected");
            return Ok(String::new());
        }

        super::reading_order::sort_reading_order(&mut boxes, img.height(), img.width());

        let mut results = Vec::with_capacity(boxes.len());
        for bbox in boxes {
            let line_img = crop_bbox(img, &bbox);
            let text = self.recognizer.recognize(&line_img, bbox.pred_char_cnt);
            if !text.is_empty() {
                results.push(text);
            }
        }

        let result = results.join("\n");
        info!(result_length = result.len(), "OCR completed");
        Ok(result)
    }
}

fn crop_bbox(image: &DynamicImage, bbox: &super::deim::BoundingBox) -> DynamicImage {
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
