use anyhow::Result;
use image::DynamicImage;
use std::path::Path;

use super::parseq::ParseqRecognizer;
use super::vocab::Vocabulary;

const CASCADE_UPGRADE_THRESHOLD_50: usize = 25;
const CASCADE_UPGRADE_THRESHOLD_100: usize = 45;

pub struct CascadeRecognizer {
    rec30: ParseqRecognizer,
    rec50: ParseqRecognizer,
    rec100: ParseqRecognizer,
}

impl CascadeRecognizer {
    pub fn new(
        model30_path: &Path,
        model50_path: &Path,
        model100_path: &Path,
        vocab_path: &Path,
    ) -> Result<Self> {
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
        let initial_rec = self.select_initial_model(pred_char_cnt);
        let text = initial_rec.read(line_img);
        self.apply_fallback(line_img, text)
    }

    fn select_initial_model(&self, pred_char_cnt: f32) -> &ParseqRecognizer {
        let rounded = pred_char_cnt.round() as i32;
        match rounded {
            3 => &self.rec30,
            2 => &self.rec50,
            _ => &self.rec100,
        }
    }

    fn apply_fallback(&self, line_img: &DynamicImage, initial_text: String) -> String {
        if initial_text.len() >= CASCADE_UPGRADE_THRESHOLD_100 {
            self.rec100.read(line_img)
        } else if initial_text.len() >= CASCADE_UPGRADE_THRESHOLD_50 {
            self.rec50.read(line_img)
        } else {
            initial_text
        }
    }
}
