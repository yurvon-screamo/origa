use std::collections::HashMap;

use candle_core::{Device, IndexOp, Tensor};
use candle_onnx::onnx::ModelProto;
use candle_onnx::simple_eval;
use image::{DynamicImage, imageops::FilterType};
use prost::Message;
use tokenizers::Tokenizer;
use tracing::{debug, info};

use crate::domain::OrigaError;

const MAX_SEQ_LEN: usize = 300;
const IMAGE_RESIZE_W: u32 = 224;
const IMAGE_RESIZE_H: u32 = 224;
const PIXEL_NORM_FACTOR: f32 = 255.0;
const NORM_MEAN: [f32; 3] = [0.5, 0.5, 0.5];
const NORM_STD: [f32; 3] = [0.5, 0.5, 0.5];
const DEFAULT_BOS_TOKEN_ID: u32 = 2;
const DEFAULT_EOS_TOKEN_ID: u32 = 3;

pub struct ModelFiles {
    pub encoder: Vec<u8>,
    pub decoder: Vec<u8>,
    pub tokenizer: Vec<u8>,
}

pub struct JapaneseOCRModel {
    encoder: ModelProto,
    decoder: ModelProto,
    tokenizer: Tokenizer,
    device: Device,
}

impl JapaneseOCRModel {
    pub fn from_bytes(
        encoder_bytes: Vec<u8>,
        decoder_bytes: Vec<u8>,
        tokenizer_bytes: Vec<u8>,
    ) -> Result<Self, OrigaError> {
        info!("Initializing Japanese OCR model");
        let device = Device::Cpu;

        debug!("Loading tokenizer");
        let tokenizer =
            Tokenizer::from_bytes(&tokenizer_bytes).map_err(|e| OrigaError::OcrError {
                reason: format!("Failed to load tokenizer: {}", e),
            })?;

        debug!("Decoding encoder ModelProto");
        let encoder =
            ModelProto::decode(encoder_bytes.as_slice()).map_err(|e| OrigaError::OcrError {
                reason: format!("Failed to decode encoder: {}", e),
            })?;

        debug!("Decoding decoder ModelProto");
        let decoder =
            ModelProto::decode(decoder_bytes.as_slice()).map_err(|e| OrigaError::OcrError {
                reason: format!("Failed to decode decoder: {}", e),
            })?;

        info!("Japanese OCR model initialized successfully");
        Ok(Self {
            encoder,
            decoder,
            tokenizer,
            device,
        })
    }

    pub fn from_model_files(files: ModelFiles) -> Result<Self, OrigaError> {
        Self::from_bytes(files.encoder, files.decoder, files.tokenizer)
    }

    pub fn run(&mut self, img: &DynamicImage) -> Result<String, OrigaError> {
        info!("Running OCR on image");
        let pixel_values = self.preprocess_image(img)?;

        let mut encoder_inputs: HashMap<String, Tensor> = HashMap::new();
        encoder_inputs.insert("pixel_values".to_string(), pixel_values);

        debug!("Executing encoder");
        let encoder_outputs =
            simple_eval(&self.encoder, encoder_inputs).map_err(|e| OrigaError::OcrError {
                reason: format!("Candle error: {}", e),
            })?;

        let encoder_hidden_states = encoder_outputs
            .get("last_hidden_state")
            .ok_or_else(|| OrigaError::OcrError {
                reason: "Missing last_hidden_state from encoder".into(),
            })?
            .clone();

        let bos_token_id = self
            .tokenizer
            .token_to_id("[CLS]")
            .unwrap_or(DEFAULT_BOS_TOKEN_ID);
        let eos_token_id = self
            .tokenizer
            .token_to_id("[SEP]")
            .unwrap_or(DEFAULT_EOS_TOKEN_ID);

        let mut input_ids = vec![bos_token_id as i64];

        for i in 0..MAX_SEQ_LEN {
            debug!(iteration = i, "Starting decoder iteration");

            let input_tensor = Tensor::from_slice(&input_ids, (1, input_ids.len()), &self.device)
                .map_err(|e| OrigaError::OcrError {
                reason: format!("Candle error: {}", e),
            })?;

            let mut decoder_inputs: HashMap<String, Tensor> = HashMap::new();
            decoder_inputs.insert("input_ids".to_string(), input_tensor);
            decoder_inputs.insert(
                "encoder_hidden_states".to_string(),
                encoder_hidden_states.clone(),
            );

            let decoder_outputs =
                simple_eval(&self.decoder, decoder_inputs).map_err(|e| OrigaError::OcrError {
                    reason: format!("Candle error: {}", e),
                })?;

            let logits = decoder_outputs
                .get("logits")
                .ok_or_else(|| OrigaError::OcrError {
                    reason: "Missing logits from decoder".into(),
                })?;

            let seq_len = logits.dim(1).map_err(|e| OrigaError::OcrError {
                reason: format!("Candle error: {}", e),
            })?;
            let last_logits = logits
                .i((0, seq_len - 1, ..))
                .map_err(|e| OrigaError::OcrError {
                    reason: format!("Candle error: {}", e),
                })?;
            let next_token = last_logits
                .argmax(0)
                .and_then(|t| t.to_scalar::<u32>())
                .map_err(|e| OrigaError::OcrError {
                    reason: format!("Candle error: {}", e),
                })?;

            if next_token == eos_token_id {
                debug!("EOS token reached at iteration {}", i);
                break;
            }

            input_ids.push(next_token as i64);
        }

        let decoded = self
            .tokenizer
            .decode(
                &input_ids.iter().map(|&id| id as u32).collect::<Vec<_>>(),
                true,
            )
            .map_err(|e| OrigaError::OcrError {
                reason: format!("Failed to decode: {}", e),
            })?;

        let result = decoded.replace(' ', "");
        info!(result = %result, "OCR completed");
        Ok(result)
    }

    fn preprocess_image(&self, img: &DynamicImage) -> Result<Tensor, OrigaError> {
        let (orig_w, orig_h) = (img.width(), img.height());
        debug!(
            width = orig_w,
            height = orig_h,
            "Preprocessing image for OCR"
        );
        let resized = img.resize_exact(IMAGE_RESIZE_W, IMAGE_RESIZE_H, FilterType::Nearest);
        let rgb = resized.to_rgb8();
        let (width, height) = rgb.dimensions();

        let mut data = Vec::with_capacity((width * height * 3) as usize);
        for pixel in rgb.pixels() {
            data.push(pixel[0] as f32 / PIXEL_NORM_FACTOR);
            data.push(pixel[1] as f32 / PIXEL_NORM_FACTOR);
            data.push(pixel[2] as f32 / PIXEL_NORM_FACTOR);
        }

        let tensor = Tensor::from_vec(data, (height as usize, width as usize, 3), &self.device)
            .map_err(|e| OrigaError::OcrError {
                reason: format!("Candle error: {}", e),
            })?;
        let mean = Tensor::new(&NORM_MEAN, &self.device).map_err(|e| OrigaError::OcrError {
            reason: format!("Candle error: {}", e),
        })?;
        let std = Tensor::new(&NORM_STD, &self.device).map_err(|e| OrigaError::OcrError {
            reason: format!("Candle error: {}", e),
        })?;

        let normalized = tensor
            .broadcast_sub(&mean)
            .and_then(|t| t.broadcast_div(&std))
            .map_err(|e| OrigaError::OcrError {
                reason: format!("Candle error: {}", e),
            })?;
        let pixel_values = normalized
            .permute((2, 0, 1))
            .and_then(|t| t.unsqueeze(0))
            .map_err(|e| OrigaError::OcrError {
                reason: format!("Candle error: {}", e),
            })?;

        Ok(pixel_values)
    }
}
