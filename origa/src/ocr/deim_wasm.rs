use super::shared::{DEIM_DEFAULT_INPUT_SIZE, deim_postprocess, deim_preprocess};
use super::types::BoundingBox;
use crate::domain::OrigaError;
use crate::ort_init;
use futures::lock::Mutex;
use image::DynamicImage;
use ort::ep::WebGPU;
use ort::session::Session;
use ort_web::ValueExt;

pub struct DeimDetector {
    session: Mutex<Session>,
    input_size: u32,
}

impl DeimDetector {
    pub async fn new(model_bytes: &[u8]) -> Result<Self, OrigaError> {
        let init = ort_init::ensure().await?;

        let builder = Session::builder().map_err(|e| OrigaError::OcrError {
            reason: format!("Failed to create session builder: {e:?}"),
        })?;

        let mut builder = if init.webgpu_active {
            builder
                .with_execution_providers([WebGPU::default().build()])
                .map_err(|e| OrigaError::OcrError {
                    reason: format!("Failed to register WebGPU EP: {e:?}"),
                })?
        } else {
            builder
        };

        let session =
            builder
                .commit_from_memory(model_bytes)
                .await
                .map_err(|e| OrigaError::OcrError {
                    reason: format!("Failed to load DEIM model: {e:?}"),
                })?;

        let input_info = session
            .inputs()
            .first()
            .ok_or_else(|| OrigaError::OcrError {
                reason: "Model has no inputs".to_string(),
            })?;

        let input_size = match input_info.dtype() {
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

    pub async fn detect(&self, image: &DynamicImage) -> Result<Vec<BoundingBox>, OrigaError> {
        let max_wh = image.height().max(image.width());

        let (input_array, scale) = deim_preprocess(image, max_wh, self.input_size)?;

        let shape: Vec<usize> = input_array.shape().to_vec();
        let data: Vec<f32> = input_array.into_raw_vec_and_offset().0;
        let input_tensor = ort::value::Tensor::from_array((shape, data))
            .map_err(|e| OrigaError::OcrError {
                reason: format!("Failed to create input tensor: {:?}", e),
            })?
            .into_dyn();

        let dims_shape = vec![1usize, 2];
        let dims_data = vec![self.input_size as i64, self.input_size as i64];
        let dims_tensor = ort::value::Tensor::from_array((dims_shape, dims_data))
            .map_err(|e| OrigaError::OcrError {
                reason: format!("Failed to create dims tensor: {:?}", e),
            })?
            .into_dyn();

        let boxes = {
            let run_options =
                ort::session::RunOptions::new().map_err(|e| OrigaError::OcrError {
                    reason: format!("Failed to create run options: {:?}", e),
                })?;
            let mut session = self.session.lock().await;
            let mut outputs = session
                .run_async(
                    ort::inputs![
                        "images" => input_tensor,
                        "orig_target_sizes" => dims_tensor
                    ],
                    &run_options,
                )
                .await
                .map_err(|e| OrigaError::OcrError {
                    reason: format!("DEIM inference failed: {:?}", e),
                })?;

            for (_name, mut output) in outputs.iter_mut() {
                output
                    .sync(ort_web::SyncDirection::Rust)
                    .await
                    .map_err(|e| OrigaError::OcrError {
                        reason: format!("Failed to sync output: {:?}", e),
                    })?;
            }

            deim_postprocess(&outputs, scale)?
        };

        Ok(boxes)
    }
}
