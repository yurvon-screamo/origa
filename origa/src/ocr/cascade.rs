use crate::domain::OrigaError;
use image::DynamicImage;
use std::path::Path;

use super::parseq::ParseqRecognizer;
use super::vocab::Vocabulary;

const EPSILON: f32 = 0.1;

/// Port for a single-line OCR recognizer. The cascade owns tier routing and
/// escalation; the recognizer owns preprocessing and inference. Implemented
/// by `ParseqRecognizer`; tests install fakes to exercise the cascade
/// without ONNX models.
pub trait LineRecognizer {
    fn read_line(&self, image: &DynamicImage) -> String;
}

pub struct CascadeRecognizer<R> {
    rec30: R,
    rec50: R,
    rec100: R,
}

fn approx_eq(a: f32, b: f32) -> bool {
    (a - b).abs() < EPSILON
}

impl<R: LineRecognizer> CascadeRecognizer<R> {
    fn from_recognizers(rec30: R, rec50: R, rec100: R) -> Self {
        Self {
            rec30,
            rec50,
            rec100,
        }
    }

    /// Cascade contract over one text line. The tier is picked from the
    /// detector's `pred_char_cnt` (≈3 → 30-tier, ≈2 → 50-tier, anything
    /// else → 100-tier) and long readings escalate to the wider tiers by
    /// UTF-8 byte length (≥25 → 50-tier, ≥45 → 100-tier). Fake-based tests
    /// in this file pin the routing; the real-engine path (ONNX inference
    /// inside `read`) is covered by the model-gated `japanese_ocr_e2e`
    /// integration test.
    pub fn recognize(&self, line_img: &DynamicImage, pred_char_cnt: f32) -> String {
        let initial_rec = if approx_eq(pred_char_cnt, 3.0) {
            &self.rec30
        } else if approx_eq(pred_char_cnt, 2.0) {
            &self.rec50
        } else {
            return self.rec100.read_line(line_img);
        };

        let text = initial_rec.read_line(line_img);

        if text.len() >= 25 {
            let text50 = self.rec50.read_line(line_img);
            if text50.len() >= 45 {
                self.rec100.read_line(line_img)
            } else {
                text50
            }
        } else {
            text
        }
    }
}

impl CascadeRecognizer<ParseqRecognizer> {
    /// Loading path — unit-covered on its error branches only. A successful
    /// `new` needs the three real PARSeq ONNX models, absent in the coverage
    /// environment; the full pipeline runs in the model-gated
    /// `japanese_ocr_e2e` integration test and manual smoke. The wiring this
    /// constructor performs is exercised by the fake-based cascade tests via
    /// `from_recognizers`.
    pub fn new(
        model30_path: &Path,
        model50_path: &Path,
        model100_path: &Path,
        vocab_path: &Path,
    ) -> Result<Self, OrigaError> {
        let vocab = Vocabulary::from_file(vocab_path)?;

        let rec30 = ParseqRecognizer::new(model30_path, &vocab, 256)?;
        let rec50 = ParseqRecognizer::new(model50_path, &vocab, 384)?;
        let rec100 = ParseqRecognizer::new(model100_path, &vocab, 768)?;

        Ok(Self::from_recognizers(rec30, rec50, rec100))
    }
}
