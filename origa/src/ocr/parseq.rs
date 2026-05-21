use super::shared::{parseq_postprocess, parseq_preprocess};
use super::vocab::Vocabulary;
use crate::domain::OrigaError;
use image::DynamicImage;
use ort::session::{Session, builder::GraphOptimizationLevel};
use ort::value::Value;
use std::path::Path;
use std::sync::Mutex;

pub struct ParseqRecognizer {
    session: Mutex<Session>,
    vocab: Vocabulary,
    input_width: u32,
}

impl ParseqRecognizer {
    pub fn new(
        model_path: &Path,
        vocab: &Vocabulary,
        input_width: u32,
    ) -> Result<Self, OrigaError> {
        let builder = Session::builder().map_err(|e| OrigaError::OcrError {
            reason: format!("Failed to create session builder: {:?}", e),
        })?;
        let mut builder = builder
            .with_optimization_level(GraphOptimizationLevel::Level3)
            .map_err(|e| OrigaError::OcrError {
                reason: format!("Failed to set optimization level: {:?}", e),
            })?;
        let session = builder
            .commit_from_file(model_path)
            .map_err(|e| OrigaError::OcrError {
                reason: format!("Failed to load PARSeq model from {:?}: {:?}", model_path, e),
            })?;

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

        let input_array = match parseq_preprocess(image, self.input_width) {
            Ok(t) => t,
            Err(e) => {
                tracing::warn!("PARSeq preprocessing failed: {}", e);
                return String::new();
            },
        };

        let input_tensor = match Value::from_array(input_array) {
            Ok(t) => t,
            Err(e) => {
                tracing::warn!("Failed to create tensor: {}", e);
                return String::new();
            },
        };

        match self.session.lock() {
            Ok(mut session) => match session.run(ort::inputs!["images" => input_tensor]) {
                Ok(outputs) => parseq_postprocess(&outputs, &self.vocab),
                Err(e) => {
                    tracing::warn!("PARSeq inference failed: {:?}", e);
                    String::new()
                },
            },
            Err(e) => {
                tracing::warn!("Session lock failed: {:?}", e);
                String::new()
            },
        }
    }
}
