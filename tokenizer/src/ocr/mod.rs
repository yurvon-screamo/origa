pub mod cascade;
pub mod deim;
pub mod parseq;
pub mod reading_order;
pub mod vocab;

pub use cascade::CascadeRecognizer;
pub use deim::{BoundingBox, DeimDetector};
pub use parseq::ParseqRecognizer;
pub use reading_order::sort_reading_order;
pub use vocab::Vocabulary;

use anyhow::Result;
use image::DynamicImage;

pub struct OcrEngine {
    detector: DeimDetector,
    recognizer: CascadeRecognizer,
}

impl OcrEngine {
    pub fn new(
        detector_model: &std::path::Path,
        rec_models: (&std::path::Path, &std::path::Path, &std::path::Path),
        vocab: &std::path::Path,
    ) -> Result<Self> {
        let detector = DeimDetector::new(detector_model)?;
        let recognizer = CascadeRecognizer::new(rec_models.0, rec_models.1, rec_models.2, vocab)?;
        Ok(Self {
            detector,
            recognizer,
        })
    }

    pub fn recognize(&self, image: &DynamicImage) -> Result<String> {
        let mut boxes = self.detector.detect(image)?;
        if boxes.is_empty() {
            return Ok(String::new());
        }

        sort_reading_order(&mut boxes, image.height(), image.width());

        let mut results = Vec::with_capacity(boxes.len());
        for bbox in boxes {
            let line_img = crop_bbox(image, &bbox);
            let text = self.recognizer.recognize(&line_img, bbox.pred_char_cnt);
            results.push(text);
        }

        Ok(results.join("\n"))
    }
}

fn crop_bbox(image: &DynamicImage, bbox: &BoundingBox) -> DynamicImage {
    let x0 = bbox.x0.max(0) as u32;
    let y0 = bbox.y0.max(0) as u32;
    let x1 = bbox.x1.max(0) as u32;
    let y1 = bbox.y1.max(0) as u32;

    let width = (x1 - x0).min(image.width() - x0);
    let height = (y1 - y0).min(image.height() - y0);

    image.crop_imm(x0, y0, width, height)
}
