use super::shared::{DEIM_DEFAULT_INPUT_SIZE, deim_postprocess, deim_preprocess};
use super::types::BoundingBox;
use crate::domain::OrigaError;
use image::DynamicImage;
use ort::session::{Session, builder::GraphOptimizationLevel};
use ort::value::Value;
use std::path::Path;
use std::sync::Mutex;

pub struct DeimDetector {
    session: Mutex<Session>,
    input_size: u32,
}

impl DeimDetector {
    pub fn new(model_path: &Path) -> Result<Self, OrigaError> {
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
                reason: format!("Failed to load DEIM model from {:?}: {:?}", model_path, e),
            })?;

        let input_shape = session
            .inputs()
            .first()
            .ok_or_else(|| OrigaError::OcrError {
                reason: "Model has no inputs".to_string(),
            })?
            .dtype();

        let input_size = match input_shape {
            ort::value::ValueType::Tensor { shape, .. } => {
                tracing::info!(?shape, "DEIM model input shape");
                if shape.len() >= 4 && shape[2] > 0 {
                    shape[2] as u32
                } else {
                    tracing::warn!(
                        ?shape,
                        fallback = DEIM_DEFAULT_INPUT_SIZE,
                        "DEIM input has dynamic or invalid dimensions, using fallback"
                    );
                    DEIM_DEFAULT_INPUT_SIZE
                }
            },
            _ => {
                tracing::warn!("DEIM input is not a tensor, using fallback input_size");
                DEIM_DEFAULT_INPUT_SIZE
            },
        };

        Ok(Self {
            session: Mutex::new(session),
            input_size,
        })
    }

    pub fn detect(&self, image: &DynamicImage) -> Result<Vec<BoundingBox>, OrigaError> {
        let max_wh = image.height().max(image.width());

        let (input_array, scale) = deim_preprocess(image, max_wh, self.input_size)?;

        let input_tensor = Value::from_array(input_array).map_err(|e| OrigaError::OcrError {
            reason: format!("Failed to create input tensor: {:?}", e),
        })?;

        let dims_array = ndarray::Array::from_shape_vec(
            (1, 2),
            vec![self.input_size as i64, self.input_size as i64],
        )
        .map_err(|e| OrigaError::OcrError {
            reason: format!("Failed to create dims array: {:?}", e),
        })?;
        let dims_tensor = Value::from_array(dims_array).map_err(|e| OrigaError::OcrError {
            reason: format!("Failed to create dims tensor: {:?}", e),
        })?;

        let boxes = {
            let mut session = self.session.lock().map_err(|e| OrigaError::OcrError {
                reason: format!("Session lock failed: {:?}", e),
            })?;
            let outputs = session
                .run(ort::inputs![
                    "images" => input_tensor,
                    "orig_target_sizes" => dims_tensor
                ])
                .map_err(|e| OrigaError::OcrError {
                    reason: format!("DEIM inference failed: {:?}", e),
                })?;
            deim_postprocess(&outputs, scale)?
        };

        Ok(boxes)
    }
}
