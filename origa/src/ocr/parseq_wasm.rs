use super::shared::{parseq_postprocess, parseq_preprocess};
use super::vocab::Vocabulary;
use crate::domain::OrigaError;
use crate::ort_init;
use futures::lock::Mutex;
use image::DynamicImage;
use ort::ep::WebGPU;
use ort::session::Session;
use ort_web::ValueExt;

pub struct ParseqRecognizer {
    session: Mutex<Session>,
    vocab: Vocabulary,
    input_width: u32,
}

impl ParseqRecognizer {
    pub async fn new(
        model_bytes: &[u8],
        vocab: &Vocabulary,
        input_width: u32,
    ) -> Result<Self, OrigaError> {
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
                    reason: format!("Failed to load PARSeq model: {e:?}"),
                })?;

        Ok(Self {
            session: Mutex::new(session),
            vocab: vocab.clone(),
            input_width,
        })
    }

    pub async fn read(&self, image: &DynamicImage) -> String {
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

        let shape: Vec<usize> = input_array.shape().to_vec();
        let data: Vec<f32> = input_array.into_raw_vec_and_offset().0;

        let input_tensor = match ort::value::Tensor::from_array((shape, data)) {
            Ok(t) => t.into_dyn(),
            Err(e) => {
                tracing::warn!("Failed to create tensor: {:?}", e);
                return String::new();
            },
        };

        let mut session = self.session.lock().await;
        let run_options = match ort::session::RunOptions::new() {
            Ok(o) => o,
            Err(e) => {
                tracing::warn!("Failed to create run options: {:?}", e);
                return String::new();
            },
        };
        let mut outputs = match session
            .run_async(ort::inputs!["images" => input_tensor], &run_options)
            .await
        {
            Ok(o) => o,
            Err(e) => {
                tracing::warn!("PARSeq inference failed: {:?}", e);
                return String::new();
            },
        };

        for (_name, mut output) in outputs.iter_mut() {
            if let Err(e) = output.sync(ort_web::SyncDirection::Rust).await {
                tracing::warn!("Failed to sync output: {:?}", e);
                return String::new();
            }
        }

        parseq_postprocess(&outputs, &self.vocab)
    }
}
