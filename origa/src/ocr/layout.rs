use candle_core::{Device, IndexOp, Tensor};
use candle_onnx::onnx::ModelProto;
use image::{DynamicImage, GenericImageView, Pixel};
use prost::Message;
use tracing::{debug, info};

use crate::domain::OrigaError;

const IMAGE_SIZE: usize = 1024;
const CONFIDENCE_THRESHOLD: f32 = 0.25;
const IOU_THRESHOLD: f32 = 0.45;

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum LayoutClass {
    Text = 0,
    Title = 1,
    Figure = 2,
    Table = 3,
    Caption = 4,
    TableCaption = 5,
    TableFootnote = 6,
    IsolateFormula = 7,
    FormulaCaption = 8,
}

impl LayoutClass {
    pub fn from_id(id: usize) -> Option<Self> {
        match id {
            0 => Some(Self::Text),
            1 => Some(Self::Title),
            2 => Some(Self::Figure),
            3 => Some(Self::Table),
            4 => Some(Self::Caption),
            5 => Some(Self::TableCaption),
            6 => Some(Self::TableFootnote),
            7 => Some(Self::IsolateFormula),
            8 => Some(Self::FormulaCaption),
            _ => None,
        }
    }

    pub fn is_text_like(&self) -> bool {
        matches!(
            self,
            Self::Text
                | Self::Title
                | Self::Caption
                | Self::TableCaption
                | Self::TableFootnote
                | Self::IsolateFormula
                | Self::FormulaCaption
        )
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BoundingBox {
    pub xmin: f32,
    pub ymin: f32,
    pub xmax: f32,
    pub ymax: f32,
    pub confidence: f32,
    pub class: LayoutClass,
}

impl BoundingBox {
    pub fn area(&self) -> f32 {
        (self.xmax - self.xmin).max(0.0) * (self.ymax - self.ymin).max(0.0)
    }

    pub fn center(&self) -> (f32, f32) {
        ((self.xmin + self.xmax) / 2.0, (self.ymin + self.ymax) / 2.0)
    }
}

pub struct LayoutModel {
    model: ModelProto,
    device: Device,
    confidence_threshold: f32,
    iou_threshold: f32,
}

impl LayoutModel {
    pub fn from_bytes(model_bytes: Vec<u8>) -> Result<Self, OrigaError> {
        info!("Loading DocLayout-YOLO model");
        let device = Device::Cpu;

        let model =
            ModelProto::decode(model_bytes.as_slice()).map_err(|e| OrigaError::OcrError {
                reason: format!("Failed to decode DocLayout-YOLO model: {}", e),
            })?;

        info!("DocLayout-YOLO model loaded successfully");
        Ok(Self {
            model,
            device,
            confidence_threshold: CONFIDENCE_THRESHOLD,
            iou_threshold: IOU_THRESHOLD,
        })
    }

    pub fn to_bytes(&self) -> Result<Vec<u8>, OrigaError> {
        let mut bytes = Vec::new();
        self.model
            .encode(&mut bytes)
            .map_err(|e| OrigaError::OcrError {
                reason: format!("Failed to encode DocLayout-YOLO model: {}", e),
            })?;
        Ok(bytes)
    }

    pub(crate) fn run_layout_model(
        &self,
        img: &DynamicImage,
    ) -> Result<Vec<BoundingBox>, OrigaError> {
        info!("Running layout analysis on image");
        let (orig_width, orig_height) = (img.width(), img.height());
        debug!(
            width = orig_width,
            height = orig_height,
            "Original image size"
        );

        let (input_tensor, scale, pad_x, pad_y) = self.preprocess_image(img)?;

        let mut inputs = std::collections::HashMap::new();
        inputs.insert("images".to_string(), input_tensor);

        debug!("Executing layout model inference");
        let outputs =
            candle_onnx::simple_eval(&self.model, inputs).map_err(|e| OrigaError::OcrError {
                reason: format!("Layout model inference failed: {}", e),
            })?;

        let output = outputs
            .get("output0")
            .ok_or_else(|| OrigaError::OcrError {
                reason: "Missing output0 from layout model".into(),
            })?
            .clone();

        let bboxes = self.postprocess(&output, orig_width, orig_height, scale, pad_x, pad_y)?;

        info!(count = bboxes.len(), "Layout analysis completed");
        Ok(bboxes)
    }

    fn preprocess_image(&self, img: &DynamicImage) -> Result<(Tensor, f32, u32, u32), OrigaError> {
        let (orig_width, orig_height) = (img.width(), img.height());

        let scale = (IMAGE_SIZE as f32 / orig_width.max(orig_height) as f32).min(1.0);
        let new_width = (orig_width as f32 * scale) as u32;
        let new_height = (orig_height as f32 * scale) as u32;

        let resized =
            img.resize_exact(new_width, new_height, image::imageops::FilterType::Lanczos3);

        let pad_x = (IMAGE_SIZE as u32 - new_width) / 2;
        let pad_y = (IMAGE_SIZE as u32 - new_height) / 2;

        let mut padded = image::RgbImage::new(IMAGE_SIZE as u32, IMAGE_SIZE as u32);
        for y in 0..new_height {
            for x in 0..new_width {
                padded[(x + pad_x, y + pad_y)] = resized.get_pixel(x, y).to_rgb();
            }
        }

        let mut data = Vec::with_capacity(3 * IMAGE_SIZE * IMAGE_SIZE);
        for pixel in padded.pixels() {
            data.push(pixel[0] as f32 / 255.0);
            data.push(pixel[1] as f32 / 255.0);
            data.push(pixel[2] as f32 / 255.0);
        }

        let tensor = Tensor::from_vec(data, (IMAGE_SIZE, IMAGE_SIZE, 3), &self.device)
            .map_err(|e| OrigaError::OcrError {
                reason: format!("Failed to create tensor: {}", e),
            })?
            .permute((2, 0, 1))
            .map_err(|e| OrigaError::OcrError {
                reason: format!("Failed to permute tensor: {}", e),
            })?
            .unsqueeze(0)
            .map_err(|e| OrigaError::OcrError {
                reason: format!("Failed to unsqueeze tensor: {}", e),
            })?;

        Ok((tensor, scale, pad_x, pad_y))
    }

    fn postprocess(
        &self,
        output: &Tensor,
        orig_width: u32,
        orig_height: u32,
        scale: f32,
        pad_x: u32,
        pad_y: u32,
    ) -> Result<Vec<BoundingBox>, OrigaError> {
        let output = output
            .squeeze(0)
            .map_err(|e| OrigaError::OcrError {
                reason: format!("Failed to squeeze output: {}", e),
            })?
            .transpose(0, 1)
            .map_err(|e| OrigaError::OcrError {
                reason: format!("Failed to transpose output: {}", e),
            })?;

        let (_, num_preds) = output.dims2().map_err(|e| OrigaError::OcrError {
            reason: format!("Failed to get output dims: {}", e),
        })?;

        let mut bboxes = Vec::new();

        for i in 0..num_preds {
            let pred =
                Vec::<f32>::try_from(output.i((.., i)).map_err(|e| OrigaError::OcrError {
                    reason: format!("Failed to get prediction {}: {}", i, e),
                })?)
                .map_err(|e| OrigaError::OcrError {
                    reason: format!("Failed to convert prediction to vec: {}", e),
                })?;

            let bbox_data = &pred[0..4];
            let class_scores = &pred[4..];

            let (class_id, &confidence) = class_scores
                .iter()
                .enumerate()
                .max_by(|a, b| a.1.partial_cmp(b.1).unwrap())
                .unwrap_or((0, &0.0));

            if confidence < self.confidence_threshold {
                continue;
            }

            let class = LayoutClass::from_id(class_id).unwrap_or(LayoutClass::Text);

            let cx = (bbox_data[0] - pad_x as f32) / scale;
            let cy = (bbox_data[1] - pad_y as f32) / scale;
            let w = bbox_data[2] / scale;
            let h = bbox_data[3] / scale;

            let xmin = (cx - w / 2.0).clamp(0.0, orig_width as f32);
            let ymin = (cy - h / 2.0).clamp(0.0, orig_height as f32);
            let xmax = (cx + w / 2.0).clamp(0.0, orig_width as f32);
            let ymax = (cy + h / 2.0).clamp(0.0, orig_height as f32);

            bboxes.push(BoundingBox {
                xmin,
                ymin,
                xmax,
                ymax,
                confidence,
                class,
            });
        }

        self.non_maximum_suppression(&mut bboxes);

        bboxes.sort_by(|a, b| {
            let a_center = a.center();
            let b_center = b.center();
            let y_diff = a_center.1 - b_center.1;
            if y_diff.abs() > 20.0 {
                y_diff.partial_cmp(&0.0).unwrap()
            } else {
                a_center.0.partial_cmp(&b_center.0).unwrap()
            }
        });

        Ok(bboxes)
    }

    fn non_maximum_suppression(&self, bboxes: &mut Vec<BoundingBox>) {
        bboxes.sort_by(|a, b| b.confidence.partial_cmp(&a.confidence).unwrap());

        let mut keep = vec![true; bboxes.len()];

        for i in 0..bboxes.len() {
            if !keep[i] {
                continue;
            }

            for j in (i + 1)..bboxes.len() {
                if !keep[j] {
                    continue;
                }

                let iou = self.calculate_iou(&bboxes[i], &bboxes[j]);
                if iou > self.iou_threshold {
                    keep[j] = false;
                }
            }
        }

        let mut idx = 0;
        bboxes.retain(|_| {
            let should_keep = keep[idx];
            idx += 1;
            should_keep
        });
    }

    fn calculate_iou(&self, box1: &BoundingBox, box2: &BoundingBox) -> f32 {
        let x1 = box1.xmin.max(box2.xmin);
        let y1 = box1.ymin.max(box2.ymin);
        let x2 = box1.xmax.min(box2.xmax);
        let y2 = box1.ymax.min(box2.ymax);

        let intersection = (x2 - x1).max(0.0) * (y2 - y1).max(0.0);

        let area1 = box1.area();
        let area2 = box2.area();

        let union = area1 + area2 - intersection;

        if union > 0.0 {
            intersection / union
        } else {
            0.0
        }
    }
}
