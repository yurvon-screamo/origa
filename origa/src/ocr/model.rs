use std::collections::HashMap;

use candle_core::{D, DType, Device, IndexOp, Tensor};
use candle_onnx::onnx::ModelProto;
use candle_onnx::simple_eval;
use image::{DynamicImage, imageops::FilterType};
use prost::Message;
use tokenizers::Tokenizer;
use tracing::{debug, info};

use super::layout::{BoundingBox, LayoutModel};
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
    pub layout_model: Vec<u8>,
}

pub struct JapaneseOCRModel {
    encoder: ModelProto,
    decoder: ModelProto,
    tokenizer: Tokenizer,
    device: Device,
    layout_model: LayoutModel,
}

impl JapaneseOCRModel {
    pub fn from_bytes(
        encoder_bytes: Vec<u8>,
        decoder_bytes: Vec<u8>,
        tokenizer_bytes: Vec<u8>,
        layout_model_bytes: Vec<u8>,
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

        debug!("Loading layout model");
        let layout_model = LayoutModel::from_bytes(layout_model_bytes)?;

        info!("Japanese OCR model initialized successfully");
        Ok(Self {
            encoder,
            decoder,
            tokenizer,
            device,
            layout_model,
        })
    }

    pub fn from_model_files(files: ModelFiles) -> Result<Self, OrigaError> {
        Self::from_bytes(
            files.encoder,
            files.decoder,
            files.tokenizer,
            files.layout_model,
        )
    }

    pub(crate) fn run(&mut self, img: &DynamicImage) -> Result<String, OrigaError> {
        info!("Running OCR with layout analysis");
        let bboxes = self.layout_model.run_layout_model(img)?;

        let mut text_bboxes: Vec<_> = bboxes
            .into_iter()
            .filter(|bbox| bbox.class.is_text_like())
            .collect();

        if text_bboxes.is_empty() {
            info!("No text regions detected, running OCR on full image");
            return self.run_ocr_on_image(img);
        }

        // Сортируем боксы: сначала по ymin (строки), затем по xmin (колонки)
        text_bboxes.sort_by(|a, b| {
            a.ymin
                .partial_cmp(&b.ymin)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| {
                    a.xmin
                        .partial_cmp(&b.xmin)
                        .unwrap_or(std::cmp::Ordering::Equal)
                })
        });

        info!(
            count = text_bboxes.len(),
            "Processing text regions with batching"
        );

        // 1. Собираем все кропы и препроцессим их в вектор тензоров
        let mut crop_tensors = Vec::with_capacity(text_bboxes.len());
        for bbox in &text_bboxes {
            let cropped = self.crop_image(img, bbox)?;
            let processed = self.preprocess_image(&cropped)?;
            crop_tensors.push(processed);
        }

        // 2. Склеиваем в батч: [N, 3, 224, 224]
        let batch_tensor = Tensor::stack(&crop_tensors, 0).map_err(|e| OrigaError::OcrError {
            reason: format!("Batching error: {}", e),
        })?;

        // 3. Прогоняем через Encoder ОДИН РАЗ
        let mut encoder_inputs = HashMap::new();
        encoder_inputs.insert("pixel_values".to_string(), batch_tensor);

        debug!("Executing batched encoder");
        let encoder_outputs =
            simple_eval(&self.encoder, encoder_inputs).map_err(|e| OrigaError::OcrError {
                reason: format!("Candle error: {}", e),
            })?;

        let all_hidden_states =
            encoder_outputs
                .get("last_hidden_state")
                .ok_or_else(|| OrigaError::OcrError {
                    reason: "Missing last_hidden_state from encoder".into(),
                })?;

        // 4. Декодируем весь батч сразу
        let results = self.decode_tokens(all_hidden_states.clone())?;

        let result = results
            .into_iter()
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>()
            .join("\n");

        info!(
            result_length = result.len(),
            "OCR with layout completed (batched)"
        );
        Ok(result)
    }

    fn run_ocr_on_image(&mut self, img: &DynamicImage) -> Result<String, OrigaError> {
        info!("Running OCR on image");
        let pixel_values =
            self.preprocess_image(img)?
                .unsqueeze(0)
                .map_err(|e| OrigaError::OcrError {
                    reason: format!("Candle error: {}", e),
                })?;

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

        let results = self.decode_tokens(encoder_hidden_states)?;
        Ok(results.join("\n"))
    }

    fn decode_tokens(&self, encoder_hidden_states: Tensor) -> Result<Vec<String>, OrigaError> {
        let batch_size = encoder_hidden_states
            .dim(0)
            .map_err(|e| OrigaError::OcrError {
                reason: format!("Candle error: {}", e),
            })?;

        let bos_token_id = self
            .tokenizer
            .token_to_id("[CLS]")
            .unwrap_or(DEFAULT_BOS_TOKEN_ID);
        let eos_token_id = self
            .tokenizer
            .token_to_id("[SEP]")
            .unwrap_or(DEFAULT_EOS_TOKEN_ID);

        let mut input_ids = Tensor::from_slice(
            &vec![bos_token_id as i64; batch_size],
            (batch_size, 1),
            &self.device,
        )
        .map_err(|e| OrigaError::OcrError {
            reason: format!("Candle error: {}", e),
        })?;

        let mut finished = vec![false; batch_size];

        for i in 0..MAX_SEQ_LEN {
            debug!(iteration = i, "Starting decoder iteration");

            let mut decoder_inputs: HashMap<String, Tensor> = HashMap::new();
            decoder_inputs.insert("input_ids".to_string(), input_ids.clone());
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

            let (_b, seq_len, _v) = logits.dims3().map_err(|e| OrigaError::OcrError {
                reason: format!("Candle error: {}", e),
            })?;

            let last_logits =
                logits
                    .i((.., seq_len - 1, ..))
                    .map_err(|e| OrigaError::OcrError {
                        reason: format!("Candle error: {}", e),
                    })?;

            let next_tokens = last_logits
                .argmax(D::Minus1)
                .map_err(|e| OrigaError::OcrError {
                    reason: format!("Candle error: {}", e),
                })?;

            let next_tokens_vec =
                next_tokens
                    .to_vec1::<u32>()
                    .map_err(|e| OrigaError::OcrError {
                        reason: format!("Candle error: {}", e),
                    })?;

            let mut all_finished = true;
            for (idx, &token) in next_tokens_vec.iter().enumerate() {
                if token == eos_token_id {
                    finished[idx] = true;
                }
                if !finished[idx] {
                    all_finished = false;
                }
            }

            let next_tokens_tensor = next_tokens
                .unsqueeze(1)
                .map_err(|e| OrigaError::OcrError {
                    reason: format!("Candle error: {}", e),
                })?
                .to_dtype(DType::I64)
                .map_err(|e| OrigaError::OcrError {
                    reason: format!("Candle error: {}", e),
                })?;

            input_ids = Tensor::cat(&[&input_ids, &next_tokens_tensor], 1).map_err(|e| {
                OrigaError::OcrError {
                    reason: format!("Candle error: {}", e),
                }
            })?;

            if all_finished {
                debug!("All sequences finished at iteration {}", i);
                break;
            }
        }

        let mut results = Vec::with_capacity(batch_size);
        for b in 0..batch_size {
            let b_ids = input_ids
                .i(b)
                .map_err(|e| OrigaError::OcrError {
                    reason: format!("Candle error: {}", e),
                })?
                .to_vec1::<i64>()
                .map_err(|e| OrigaError::OcrError {
                    reason: format!("Candle error: {}", e),
                })?;

            let mut ids_to_decode = Vec::new();
            for id in b_ids {
                let id_u32 = id as u32;
                ids_to_decode.push(id_u32);
                if id_u32 == eos_token_id {
                    break;
                }
            }

            let decoded =
                self.tokenizer
                    .decode(&ids_to_decode, true)
                    .map_err(|e| OrigaError::OcrError {
                        reason: format!("Failed to decode: {}", e),
                    })?;
            results.push(decoded.replace(' ', ""));
        }

        Ok(results)
    }

    fn crop_image(
        &self,
        img: &DynamicImage,
        bbox: &BoundingBox,
    ) -> Result<DynamicImage, OrigaError> {
        let (width, height) = (img.width(), img.height());

        let xmin = bbox.xmin as u32;
        let ymin = bbox.ymin as u32;
        let xmax = (bbox.xmax as u32).min(width);
        let ymax = (bbox.ymax as u32).min(height);

        if xmax <= xmin || ymax <= ymin {
            return Err(OrigaError::OcrError {
                reason: "Invalid bounding box".into(),
            });
        }

        let crop_width = xmax - xmin;
        let crop_height = ymax - ymin;

        let cropped = img.crop_imm(xmin, ymin, crop_width, crop_height);
        Ok(cropped)
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
            .map_err(|e| OrigaError::OcrError {
                reason: format!("Candle error: {}", e),
            })?;

        Ok(pixel_values)
    }
}
