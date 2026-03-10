use crate::domain::OrigaError;
use image::DynamicImage;

use super::parseq_wasm::ParseqRecognizer;
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

#[cfg(target_arch = "wasm32")]
impl CascadeRecognizer {
    pub async fn new(
        model30_bytes: &[u8],
        model50_bytes: &[u8],
        model100_bytes: &[u8],
        vocab_bytes: &[u8],
    ) -> Result<Self, OrigaError> {
        let vocab = Vocabulary::from_bytes(vocab_bytes)?;

        let rec30 = ParseqRecognizer::new(model30_bytes, &vocab, 256).await?;
        let rec50 = ParseqRecognizer::new(model50_bytes, &vocab, 384).await?;
        let rec100 = ParseqRecognizer::new(model100_bytes, &vocab, 768).await?;

        Ok(Self {
            rec30,
            rec50,
            rec100,
        })
    }

    pub async fn recognize(&self, line_img: &DynamicImage, pred_char_cnt: f32) -> String {
        let initial_rec = if approx_eq(pred_char_cnt, 3.0) {
            &self.rec30
        } else if approx_eq(pred_char_cnt, 2.0) {
            &self.rec50
        } else {
            return self.rec100.read(line_img).await;
        };

        let text = initial_rec.read(line_img).await;

        if text.len() >= 25 {
            let text50 = self.rec50.read(line_img).await;
            if text50.len() >= 45 {
                self.rec100.read(line_img).await
            } else {
                text50
            }
        } else {
            text
        }
    }
}
