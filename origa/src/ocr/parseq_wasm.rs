use super::deim_wasm::ensure_ort_initialized;
use super::vocab::Vocabulary;
use crate::domain::OrigaError;
use as_slice::AsSlice;
use futures::lock::Mutex;
use image::DynamicImage;
use ort::session::Session;
use ort_web::ValueExt;

const INPUT_HEIGHT: u32 = 16;

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
        ensure_ort_initialized().await?;

        let mut builder = Session::builder().map_err(|e| OrigaError::OcrError {
            reason: format!("Failed to create session builder: {:?}", e),
        })?;
        let session = builder
            .commit_from_memory(model_bytes)
            .await
            .map_err(|e| OrigaError::OcrError {
                reason: format!("Failed to load PARSeq model: {:?}", e),
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

        let input_array = match self.preprocess(image) {
            Ok(t) => t,
            Err(e) => {
                tracing::warn!("PARSeq preprocessing failed: {}", e);
                return String::new();
            }
        };

        let shape: Vec<usize> = input_array.shape().to_vec();
        let data: Vec<f32> = input_array.into_raw_vec_and_offset().0;
        
        let input_tensor = match ort::value::Tensor::from_array((shape, data)) {
            Ok(t) => t.into_dyn(),
            Err(e) => {
                tracing::warn!("Failed to create tensor: {:?}", e);
                return String::new();
            }
        };

        let mut session = self.session.lock().await;
        let run_options = match ort::session::RunOptions::new() {
            Ok(o) => o,
            Err(e) => {
                tracing::warn!("Failed to create run options: {:?}", e);
                return String::new();
            }
        };
        let mut outputs = match session
            .run_async(
                ort::inputs!["images" => input_tensor],
                &run_options,
            )
            .await
        {
            Ok(o) => o,
            Err(e) => {
                tracing::warn!("PARSeq inference failed: {:?}", e);
                return String::new();
            }
        };

        for (_name, mut output) in outputs.iter_mut() {
            if let Err(e) = output.sync(ort_web::SyncDirection::Rust).await {
                tracing::warn!("Failed to sync output: {:?}", e);
                return String::new();
            }
        }

        self.postprocess(&outputs)
    }

    fn preprocess(&self, image: &DynamicImage) -> Result<ndarray::Array4<f32>, OrigaError> {
        let (mut img_w, mut img_h) = (image.width(), image.height());

        let rotated = if img_h > img_w {
            std::mem::swap(&mut img_w, &mut img_h);
            true
        } else {
            false
        };

        let target_width = self.input_width;

        let resized = if rotated {
            let rotated_img = image.rotate270();
            image::imageops::resize(
                &rotated_img,
                target_width,
                INPUT_HEIGHT,
                image::imageops::FilterType::Triangle,
            )
        } else {
            image::imageops::resize(
                image,
                target_width,
                INPUT_HEIGHT,
                image::imageops::FilterType::Triangle,
            )
        };

        let mut tensor =
            ndarray::Array4::<f32>::zeros((1, 3, INPUT_HEIGHT as usize, target_width as usize));

        for y in 0..INPUT_HEIGHT as usize {
            for x in 0..target_width as usize {
                let pixel = resized.get_pixel(x as u32, y as u32);
                for c in 0..3 {
                    let val = pixel[2 - c] as f32 / 127.5 - 1.0;
                    tensor[[0, c, y, x]] = val;
                }
            }
        }

        Ok(tensor)
    }

    fn postprocess(&self, outputs: &ort::session::SessionOutputs<'_>) -> String {
        let logits_value = &outputs[0];

        let (shape, logits_data): (&ort::value::Shape, &[f32]) = match logits_value.try_extract_tensor() {
            Ok(t) => t,
            Err(e) => {
                tracing::warn!("Failed to extract PARSeq output tensor: {:?}", e);
                return String::new();
            }
        };

        let shape_slice = shape.as_slice();

        if shape_slice.len() < 3 {
            tracing::warn!("Invalid PARSeq output shape: {:?}", shape_slice);
            return String::new();
        }

        let seq_len = shape_slice[1] as usize;
        let vocab_size = shape_slice[2] as usize;

        let mut indices = Vec::with_capacity(seq_len);

        for t in 0..seq_len {
            let mut max_idx = 0;
            let mut max_val = f32::NEG_INFINITY;
            for v in 0..vocab_size {
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
