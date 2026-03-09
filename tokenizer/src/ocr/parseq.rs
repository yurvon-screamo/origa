use anyhow::{Context, Result};
use image::DynamicImage;
use ort::session::{builder::GraphOptimizationLevel, Session, SessionOutputs};
use ort::value::Value;
use std::path::Path;
use std::sync::Mutex;

use super::vocab::Vocabulary;

const PARSEQ_INPUT_HEIGHT: u32 = 16;
const PARSEQ_NORMALIZATION_SCALE: f32 = 127.5;
const PARSEQ_NORMALIZATION_OFFSET: f32 = 1.0;

pub struct ParseqRecognizer {
    session: Mutex<Session>,
    vocab: Vocabulary,
    input_width: u32,
}

impl ParseqRecognizer {
    pub fn new(model_path: &Path, vocab: &Vocabulary, input_width: u32) -> Result<Self> {
        let session = Session::builder()?
            .with_optimization_level(GraphOptimizationLevel::Level3)
            .map_err(|e| anyhow::anyhow!("Failed to set optimization level: {:?}", e))?
            .commit_from_file(model_path)
            .with_context(|| format!("Failed to load PARSeq model from {:?}", model_path))?;

        Ok(Self {
            session: Mutex::new(session),
            vocab: vocab.clone(),
            input_width,
        })
    }

    pub fn read(&self, image: &DynamicImage) -> String {
        if image.width() == 0 || image.height() == 0 {
            return String::new();
        }

        let input_array = match self.preprocess(image) {
            Ok(t) => t,
            Err(e) => {
                tracing::warn!("PARSeq preprocessing failed: {}", e);
                return String::new();
            }
        };

        let input_tensor = match Value::from_array(input_array) {
            Ok(t) => t,
            Err(e) => {
                tracing::warn!("Failed to create tensor: {}", e);
                return String::new();
            }
        };

        let result = {
            let mut session = match self.session.lock() {
                Ok(s) => s,
                Err(e) => {
                    tracing::warn!("PARSeq inference session error: {:?}", e);
                    return String::new();
                }
            };
            match session.run(ort::inputs!["images" => input_tensor]) {
                Ok(outputs) => self.postprocess(&outputs),
                Err(e) => {
                    tracing::warn!("PARSeq inference failed: {:?}", e);
                    String::new()
                }
            }
        };

        result
    }

    fn preprocess(&self, image: &DynamicImage) -> Result<ndarray::Array4<f32>> {
        let (mut img_w, mut img_h) = (image.width(), image.height());

        let rotated = if img_h > img_w {
            std::mem::swap(&mut img_w, &mut img_h);
            true
        } else {
            false
        };

        let target_height = PARSEQ_INPUT_HEIGHT;
        let target_width = self.input_width;

        let resized = if rotated {
            let rotated_img = image.rotate270();
            image::imageops::resize(
                &rotated_img,
                target_width,
                target_height,
                image::imageops::FilterType::Triangle,
            )
        } else {
            image::imageops::resize(
                image,
                target_width,
                target_height,
                image::imageops::FilterType::Triangle,
            )
        };

        let mut tensor =
            ndarray::Array4::<f32>::zeros((1, 3, target_height as usize, target_width as usize));

        for y in 0..target_height as usize {
            for x in 0..target_width as usize {
                let pixel = resized.get_pixel(x as u32, y as u32);
                for c in 0..3 {
                    let val = pixel[2 - c] as f32 / PARSEQ_NORMALIZATION_SCALE
                        - PARSEQ_NORMALIZATION_OFFSET;
                    tensor[[0, c, y, x]] = val;
                }
            }
        }

        Ok(tensor)
    }

    fn postprocess(&self, outputs: &SessionOutputs) -> String {
        let logits_value: &Value = &outputs[0];

        let (shape, logits_data): (&ort::value::Shape, &[f32]) =
            match logits_value.try_extract_tensor() {
                Ok(t) => t,
                Err(e) => {
                    tracing::warn!("Failed to extract logits tensor: {:?}", e);
                    return String::new();
                }
            };

        if shape.len() < 3 {
            tracing::warn!("Unexpected logits shape length: {}", shape.len());
            return String::new();
        }

        let seq_len = shape[1] as usize;
        let vocab_size = shape[2] as usize;

        let mut indices = Vec::with_capacity(seq_len);

        for t in 0..seq_len {
            let mut max_idx = 1;
            let mut max_val = f32::NEG_INFINITY;
            for v in 1..vocab_size {
                let val = logits_data[t * vocab_size + v];
                if val > max_val {
                    max_val = val;
                    max_idx = v;
                }
            }
            indices.push(max_idx as i64);
        }

        let end_pos = indices
            .iter()
            .position(|&idx| idx == 0)
            .unwrap_or(indices.len());

        let valid_indices: Vec<i64> = indices[..end_pos].to_vec();

        self.vocab.decode(&valid_indices)
    }
}
