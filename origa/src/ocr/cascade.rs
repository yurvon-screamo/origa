use crate::domain::OrigaError;
use image::DynamicImage;
use std::path::Path;

use super::parseq::ParseqRecognizer;
use super::vocab::Vocabulary;

const EPSILON: f32 = 0.1;

pub struct CascadeRecognizer {
    rec30: ParseqRecognizer,
    rec50: ParseqRecognizer,
    rec100: ParseqRecognizer,
}

fn approx_eq(a: f32, b: f32) -> bool {
    (a - b).abs() < EPSILON
}

impl CascadeRecognizer {
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

        Ok(Self {
            rec30,
            rec50,
            rec100,
        })
    }

    pub fn recognize(&self, line_img: &DynamicImage, pred_char_cnt: f32) -> String {
        let initial_rec = if approx_eq(pred_char_cnt, 3.0) {
            &self.rec30
        } else if approx_eq(pred_char_cnt, 2.0) {
            &self.rec50
        } else {
            return self.rec100.read(line_img);
        };

        let text = initial_rec.read(line_img);

        if text.len() >= 25 {
            let text50 = self.rec50.read(line_img);
            if text50.len() >= 45 {
                self.rec100.read(line_img)
            } else {
                text50
            }
        } else {
            text
        }
    }
}
