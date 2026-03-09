use anyhow::{Context, Result, bail};
use image::{DynamicImage, GenericImageView, ImageBuffer, Pixel, Rgb};
use ort::session::{Session, SessionOutputs, builder::GraphOptimizationLevel};
use ort::value::Value;
use std::path::Path;
use std::sync::Mutex;

/// Minimum confidence threshold for detection filtering
/// Detections below this value are discarded
const CONF_THRESHOLD: f32 = 0.25;

/// ImageNet normalization constants for DEIM preprocessing
const IMAGENET_MEAN: [f32; 3] = [0.485, 0.456, 0.406];
const IMAGENET_STD: [f32; 3] = [0.229, 0.224, 0.225];

#[derive(Debug, Clone)]
pub struct BoundingBox {
    pub x0: i32,
    pub y0: i32,
    pub x1: i32,
    pub y1: i32,
    pub confidence: f32,
    pub class_index: usize,
    pub pred_char_cnt: f32,
}

pub struct DeimDetector {
    session: Mutex<Session>,
    input_size: u32,
}

impl DeimDetector {
    pub fn new(model_path: &Path) -> Result<Self> {
        let builder = Session::builder()?;
        let mut builder = builder
            .with_optimization_level(GraphOptimizationLevel::Level3)
            .map_err(|e| anyhow::anyhow!("Failed to set optimization level: {:?}", e))?;
        let session = builder
            .commit_from_file(model_path)
            .with_context(|| format!("Failed to load DEIM model from {:?}", model_path))?;

        let input_shape = session
            .inputs()
            .first()
            .context("Model has no inputs")?
            .dtype();

        let input_size = match input_shape {
            ort::value::ValueType::Tensor { shape, .. } => {
                if shape.len() >= 4 {
                    shape[2] as u32
                } else {
                    1024
                }
            }
            _ => 1024,
        };

        Ok(Self {
            session: Mutex::new(session),
            input_size,
        })
    }

    pub fn detect(&self, image: &DynamicImage) -> Result<Vec<BoundingBox>> {
        let (img_h, img_w) = (image.height(), image.width());
        let max_wh = img_h.max(img_w);

        let (input_array, scale) = self.preprocess(image, max_wh)?;

        let input_tensor =
            Value::from_array(input_array).context("Failed to create input tensor")?;

        let dims_array = ndarray::Array::from_shape_vec(
            (1, 2),
            vec![self.input_size as i64, self.input_size as i64],
        )?;
        let dims_tensor = Value::from_array(dims_array).context("Failed to create dims tensor")?;

        let boxes = {
            let mut session = self
                .session
                .lock()
                .map_err(|e| anyhow::anyhow!("Session lock failed: {:?}", e))?;
            let outputs = session
                .run(ort::inputs![
                    "images" => input_tensor,
                    "orig_target_sizes" => dims_tensor
                ])
                .context("DEIM inference failed")?;
            self.postprocess(&outputs, scale)?
        };

        Ok(boxes)
    }

    fn preprocess(&self, image: &DynamicImage, max_wh: u32) -> Result<(ndarray::Array4<f32>, f32)> {
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

    fn postprocess(&self, outputs: &SessionOutputs, scale: f32) -> Result<Vec<BoundingBox>> {
        let mut detections = Vec::new();

        let boxes_value: &Value = &outputs["boxes"];
        let scores_value: &Value = &outputs["scores"];
        let labels_value: &Value = &outputs["labels"];

        let (_boxes_shape, boxes_data): (&ort::value::Shape, &[f32]) = boxes_value
            .try_extract_tensor()
            .context("Failed to extract boxes")?;
        let (scores_shape, scores_data): (&ort::value::Shape, &[f32]) = scores_value
            .try_extract_tensor()
            .context("Failed to extract scores")?;
        let (_labels_shape, labels_data): (&ort::value::Shape, &[i64]) = labels_value
            .try_extract_tensor()
            .context("Failed to extract labels")?;

        if scores_shape.len() < 2 {
            return Ok(detections);
        }

        let num_detections = scores_shape[1] as usize;

        if num_detections == 0 {
            return Ok(detections);
        }

        let expected_boxes_len = num_detections * 4;
        if boxes_data.len() < expected_boxes_len {
            bail!(
                "Boxes tensor too small: {} < {}",
                boxes_data.len(),
                expected_boxes_len
            );
        }
        if scores_data.len() < num_detections {
            bail!(
                "Scores tensor too small: {} < {}",
                scores_data.len(),
                num_detections
            );
        }
        if labels_data.len() < num_detections {
            bail!(
                "Labels tensor too small: {} < {}",
                labels_data.len(),
                num_detections
            );
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
