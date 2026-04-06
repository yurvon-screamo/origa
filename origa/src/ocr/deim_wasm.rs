use super::types::BoundingBox;
use crate::domain::OrigaError;
use futures::lock::Mutex;
use image::{DynamicImage, GenericImageView, ImageBuffer, Pixel, Rgb};
use ort::session::Session;
use ort_web::ValueExt;

const CONF_THRESHOLD: f32 = 0.25;

const IMAGENET_MEAN: [f32; 3] = [0.485, 0.456, 0.406];
const IMAGENET_STD: [f32; 3] = [0.229, 0.224, 0.225];

pub struct DeimDetector {
    session: Mutex<Session>,
    input_size: u32,
}

static ORT_INIT: std::sync::Once = std::sync::Once::new();

pub async fn ensure_ort_initialized() -> Result<(), OrigaError> {
    let mut should_init = false;
    ORT_INIT.call_once(|| {
        should_init = true;
    });

    if should_init {
        let api =
            ort_web::api(ort_web::FEATURE_WEBGPU)
                .await
                .map_err(|e| OrigaError::OcrError {
                    reason: format!("Failed to get ort WebGPU API: {:?}", e),
                })?;
        ort::set_api(api);
    }

    Ok(())
}

impl DeimDetector {
    pub async fn new(model_bytes: &[u8]) -> Result<Self, OrigaError> {
        ensure_ort_initialized().await?;

        let mut builder = Session::builder().map_err(|e| OrigaError::OcrError {
            reason: format!("Failed to create session builder: {:?}", e),
        })?;
        builder = builder
            .with_execution_providers([ort::ep::WebGPU::default().build()])
            .map_err(|e| OrigaError::OcrError {
                reason: format!("Failed to set execution providers: {:?}", e),
            })?;
        let session =
            builder
                .commit_from_memory(model_bytes)
                .await
                .map_err(|e| OrigaError::OcrError {
                    reason: format!("Failed to load DEIM model: {:?}", e),
                })?;

        let input_info = session
            .inputs()
            .first()
            .ok_or_else(|| OrigaError::OcrError {
                reason: "Model has no inputs".to_string(),
            })?;

        let input_size = match input_info.dtype() {
            ort::value::ValueType::Tensor { shape, .. } => {
                if shape.len() >= 4 {
                    shape[2] as u32
                } else {
                    1024
                }
            },
            _ => 1024,
        };

        Ok(Self {
            session: Mutex::new(session),
            input_size,
        })
    }

    pub async fn detect(&self, image: &DynamicImage) -> Result<Vec<BoundingBox>, OrigaError> {
        let (img_h, img_w) = (image.height(), image.width());
        let max_wh = img_h.max(img_w);

        let (input_array, scale) = self.preprocess(image, max_wh)?;

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

            self.postprocess(&outputs, scale)?
        };

        Ok(boxes)
    }

    fn preprocess(
        &self,
        image: &DynamicImage,
        max_wh: u32,
    ) -> Result<(ndarray::Array4<f32>, f32), OrigaError> {
        let mut padded = ImageBuffer::<Rgb<u8>, Vec<u8>>::new(max_wh, max_wh);
        for (x, y, pixel) in image.pixels() {
            padded.put_pixel(x, y, pixel.to_rgb());
        }

        let resized = image::imageops::resize(
            &padded,
            self.input_size,
            self.input_size,
            image::imageops::FilterType::CatmullRom,
        );

        let scale = max_wh as f32 / self.input_size as f32;

        let mut tensor = ndarray::Array4::<f32>::zeros((
            1,
            3,
            self.input_size as usize,
            self.input_size as usize,
        ));

        for y in 0..self.input_size as usize {
            for x in 0..self.input_size as usize {
                let pixel = resized.get_pixel(x as u32, y as u32);
                for c in 0..3 {
                    let val = pixel[c] as f32 / 255.0;
                    let normalized = (val - IMAGENET_MEAN[c]) / IMAGENET_STD[c];
                    tensor[[0, c, y, x]] = normalized;
                }
            }
        }

        Ok((tensor, scale))
    }

    fn postprocess(
        &self,
        outputs: &ort::session::SessionOutputs<'_>,
        scale: f32,
    ) -> Result<Vec<BoundingBox>, OrigaError> {
        let mut detections = Vec::new();

        let boxes_value = &outputs["boxes"];
        let scores_value = &outputs["scores"];
        let labels_value = &outputs["labels"];

        let (_boxes_shape, boxes_data): (&ort::value::Shape, &[f32]) = boxes_value
            .try_extract_tensor()
            .map_err(|e| OrigaError::OcrError {
                reason: format!("Failed to extract boxes: {:?}", e),
            })?;
        let (scores_shape, scores_data): (&ort::value::Shape, &[f32]) = scores_value
            .try_extract_tensor()
            .map_err(|e| OrigaError::OcrError {
                reason: format!("Failed to extract scores: {:?}", e),
            })?;
        let (_labels_shape, labels_data): (&ort::value::Shape, &[i64]) = labels_value
            .try_extract_tensor()
            .map_err(|e| OrigaError::OcrError {
                reason: format!("Failed to extract labels: {:?}", e),
            })?;

        if scores_shape.len() < 2 {
            return Ok(detections);
        }

        let num_detections = scores_shape[1] as usize;

        if num_detections == 0 {
            return Ok(detections);
        }

        let expected_boxes_len = num_detections * 4;
        if boxes_data.len() < expected_boxes_len {
            return Err(OrigaError::OcrError {
                reason: format!(
                    "Boxes tensor too small: {} < {}",
                    boxes_data.len(),
                    expected_boxes_len
                ),
            });
        }
        if scores_data.len() < num_detections {
            return Err(OrigaError::OcrError {
                reason: format!(
                    "Scores tensor too small: {} < {}",
                    scores_data.len(),
                    num_detections
                ),
            });
        }
        if labels_data.len() < num_detections {
            return Err(OrigaError::OcrError {
                reason: format!(
                    "Labels tensor too small: {} < {}",
                    labels_data.len(),
                    num_detections
                ),
            });
        }

        let char_counts: Option<Vec<f32>> = if outputs.len() >= 4 {
            outputs.get("char_count").and_then(|v| {
                let (_shape, data): (&ort::value::Shape, &[i64]) = v.try_extract_tensor().ok()?;
                Some(data.iter().map(|&v| v as f32).collect())
            })
        } else {
            None
        };

        for i in 0..num_detections {
            let score = scores_data[i];
            if score < CONF_THRESHOLD {
                continue;
            }

            let x0 = boxes_data[i * 4] * scale;
            let y0 = boxes_data[i * 4 + 1] * scale;
            let x1 = boxes_data[i * 4 + 2] * scale;
            let y1 = boxes_data[i * 4 + 3] * scale;

            let label = labels_data[i] as usize;
            let char_count = char_counts.as_ref().map(|c| c[i]).unwrap_or(100.0);

            detections.push(BoundingBox {
                x0: x0 as i32,
                y0: y0 as i32,
                x1: x1 as i32,
                y1: y1 as i32,
                confidence: score,
                class_index: label.saturating_sub(1),
                pred_char_cnt: char_count,
            });
        }

        Ok(detections)
    }
}
