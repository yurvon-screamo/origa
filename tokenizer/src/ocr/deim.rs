use anyhow::{Context, Result};
use image::{DynamicImage, GenericImageView, ImageBuffer, Pixel, Rgb};
use ort::session::{Session, SessionOutputs, builder::GraphOptimizationLevel};
use ort::value::Value;
use std::cell::RefCell;
use std::path::Path;

const INPUT_SIZE: u32 = 800;
const CONF_THRESHOLD: f32 = 0.25;

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
    session: RefCell<Session>,
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

        Ok(Self {
            session: RefCell::new(session),
        })
    }

    pub fn detect(&self, image: &DynamicImage) -> Result<Vec<BoundingBox>> {
        let (img_h, img_w) = (image.height(), image.width());
        let max_wh = img_h.max(img_w);

        let (input_array, scale_x, scale_y) = self.preprocess(image, max_wh)?;

        let input_tensor =
            Value::from_array(input_array).context("Failed to create input tensor")?;

        let dims_array =
            ndarray::Array::from_shape_vec((1, 2), vec![INPUT_SIZE as i64, INPUT_SIZE as i64])?;
        let dims_tensor = Value::from_array(dims_array).context("Failed to create dims tensor")?;

        let boxes = {
            let mut session = self.session.borrow_mut();
            let outputs = session
                .run(ort::inputs![
                    "images" => input_tensor,
                    "orig_target_sizes" => dims_tensor
                ])
                .context("DEIM inference failed")?;
            self.postprocess(&outputs, scale_x, scale_y)?
        };

        Ok(boxes)
    }

    fn preprocess(
        &self,
        image: &DynamicImage,
        max_wh: u32,
    ) -> Result<(ndarray::Array4<f32>, f32, f32)> {
        let mut padded = ImageBuffer::<Rgb<u8>, Vec<u8>>::new(max_wh, max_wh);
        for (x, y, pixel) in image.pixels() {
            padded.put_pixel(x, y, pixel.to_rgb());
        }

        let resized = image::imageops::resize(
            &padded,
            INPUT_SIZE,
            INPUT_SIZE,
            image::imageops::FilterType::CatmullRom,
        );

        let scale_x = max_wh as f32 / INPUT_SIZE as f32;
        let scale_y = max_wh as f32 / INPUT_SIZE as f32;

        let mut tensor =
            ndarray::Array4::<f32>::zeros((1, 3, INPUT_SIZE as usize, INPUT_SIZE as usize));

        let mean = [0.485, 0.456, 0.406];
        let std = [0.229, 0.224, 0.225];

        for y in 0..INPUT_SIZE as usize {
            for x in 0..INPUT_SIZE as usize {
                let pixel = resized.get_pixel(x as u32, y as u32);
                for c in 0..3 {
                    let val = pixel[c] as f32 / 255.0;
                    let normalized = (val - mean[c]) / std[c];
                    tensor[[0, c, y, x]] = normalized;
                }
            }
        }

        Ok((tensor, scale_x, scale_y))
    }

    fn postprocess(
        &self,
        outputs: &SessionOutputs,
        scale_x: f32,
        scale_y: f32,
    ) -> Result<Vec<BoundingBox>> {
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

        let num_detections = scores_shape[1] as usize;

        let char_counts: Option<Vec<f32>> = if outputs.len() >= 4 {
            outputs.get("char_counts").and_then(|v| {
                let (_shape, data): (&ort::value::Shape, &[f32]) = v.try_extract_tensor().ok()?;
                Some(data.to_vec())
            })
        } else {
            None
        };

        for i in 0..num_detections {
            let score = scores_data[i];
            if score < CONF_THRESHOLD {
                continue;
            }

            let x0 = boxes_data[i * 4] * scale_x;
            let y0 = boxes_data[i * 4 + 1] * scale_y;
            let x1 = boxes_data[i * 4 + 2] * scale_x;
            let y1 = boxes_data[i * 4 + 3] * scale_y;

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
