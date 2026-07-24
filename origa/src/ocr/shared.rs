use crate::domain::OrigaError;
use image::{DynamicImage, GenericImageView, ImageBuffer, Pixel, Rgb};
use ort::session::SessionOutputs;

use super::types::BoundingBox;
use super::vocab::Vocabulary;

pub(crate) const DEIM_CONF_THRESHOLD: f32 = 0.25;
pub(crate) const DEIM_DEFAULT_INPUT_SIZE: u32 = 1024;
const IMAGENET_MEAN: [f32; 3] = [0.485, 0.456, 0.406];
const IMAGENET_STD: [f32; 3] = [0.229, 0.224, 0.225];

pub(crate) const PARSEQ_INPUT_HEIGHT: u32 = 16;

pub(crate) fn deim_preprocess(
    image: &DynamicImage,
    max_wh: u32,
    input_size: u32,
) -> Result<(ndarray::Array4<f32>, f32), OrigaError> {
    let mut padded = ImageBuffer::<Rgb<u8>, Vec<u8>>::new(max_wh, max_wh);
    for (x, y, pixel) in image.pixels() {
        padded.put_pixel(x, y, pixel.to_rgb());
    }

    let resized = image::imageops::resize(
        &padded,
        input_size,
        input_size,
        image::imageops::FilterType::CatmullRom,
    );

    let scale = max_wh as f32 / input_size as f32;

    let mut tensor =
        ndarray::Array4::<f32>::zeros((1, 3, input_size as usize, input_size as usize));

    for y in 0..input_size as usize {
        for x in 0..input_size as usize {
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

pub(crate) fn deim_postprocess(
    outputs: &SessionOutputs,
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
        if score < DEIM_CONF_THRESHOLD {
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

pub(crate) fn parseq_preprocess(
    image: &DynamicImage,
    input_width: u32,
) -> Result<ndarray::Array4<f32>, OrigaError> {
    let (mut img_w, mut img_h) = (image.width(), image.height());

    let rotated = if img_h > img_w {
        std::mem::swap(&mut img_w, &mut img_h);
        true
    } else {
        false
    };

    let target_width = input_width;

    let resized = if rotated {
        let rotated_img = image.rotate270();
        image::imageops::resize(
            &rotated_img,
            target_width,
            PARSEQ_INPUT_HEIGHT,
            image::imageops::FilterType::Triangle,
        )
    } else {
        image::imageops::resize(
            image,
            target_width,
            PARSEQ_INPUT_HEIGHT,
            image::imageops::FilterType::Triangle,
        )
    };

    let mut tensor =
        ndarray::Array4::<f32>::zeros((1, 3, PARSEQ_INPUT_HEIGHT as usize, target_width as usize));

    for y in 0..PARSEQ_INPUT_HEIGHT as usize {
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

pub(crate) fn parseq_postprocess(outputs: &SessionOutputs, vocab: &Vocabulary) -> String {
    let logits_value = &outputs[0];

    let (shape, logits_data): (&ort::value::Shape, &[f32]) = match logits_value.try_extract_tensor()
    {
        Ok(t) => t,
        Err(e) => {
            tracing::warn!("Failed to extract PARSeq output tensor: {:?}", e);
            return String::new();
        },
    };

    if shape.len() < 3 {
        tracing::warn!("Invalid PARSeq output shape: {:?}", shape);
        return String::new();
    }

    if shape[1] <= 0 || shape[2] <= 0 {
        tracing::warn!(?shape, "PARSeq output has dynamic or invalid dimensions");
        return String::new();
    }

    let seq_len = shape[1] as usize;
    let vocab_size = shape[2] as usize;

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

    vocab.decode(&valid_indices)
}

pub struct ModelFiles {
    pub deim: Vec<u8>,
    pub parseq30: Vec<u8>,
    pub parseq50: Vec<u8>,
    pub parseq100: Vec<u8>,
    pub vocab: Vec<u8>,
}

pub(crate) fn crop_bbox(image: &DynamicImage, bbox: &BoundingBox) -> DynamicImage {
    let x0 = bbox.x0.max(0) as u32;
    let y0 = bbox.y0.max(0) as u32;
    let x1 = bbox.x1.max(0) as u32;
    let y1 = bbox.y1.max(0) as u32;

    if x1 <= x0 || y1 <= y0 {
        return image.crop_imm(0, 0, 1, 1);
    }

    let x0 = x0.min(image.width());
    let y0 = y0.min(image.height());
    let x1 = x1.min(image.width());
    let y1 = y1.min(image.height());

    let width = x1 - x0;
    let height = y1 - y0;

    if width == 0 || height == 0 {
        return image.crop_imm(0, 0, 1, 1);
    }

    image.crop_imm(x0, y0, width, height)
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::RgbaImage;

    fn solid_image(w: u32, h: u32) -> DynamicImage {
        DynamicImage::ImageRgba8(RgbaImage::new(w, h))
    }

    fn bbox(x0: i32, y0: i32, x1: i32, y1: i32) -> BoundingBox {
        BoundingBox {
            x0,
            y0,
            x1,
            y1,
            confidence: 1.0,
            class_index: 0,
            pred_char_cnt: 0.0,
        }
    }

    #[test]
    fn crop_bbox_returns_full_crop_for_valid_coordinates() {
        let image = solid_image(100, 100);
        let cropped = crop_bbox(&image, &bbox(10, 20, 30, 40));
        assert_eq!(cropped.width(), 20);
        assert_eq!(cropped.height(), 20);
    }

    #[test]
    fn crop_bbox_clamps_negative_origin_to_zero() {
        let image = solid_image(100, 100);
        let cropped = crop_bbox(&image, &bbox(-10, -20, 30, 40));
        assert_eq!(cropped.width(), 30);
        assert_eq!(cropped.height(), 40);
    }

    #[test]
    fn crop_bbox_clamps_to_image_bounds_when_bbox_exceeds_dimensions() {
        let image = solid_image(50, 50);
        let cropped = crop_bbox(&image, &bbox(40, 40, 200, 200));
        assert_eq!(cropped.width(), 10);
        assert_eq!(cropped.height(), 10);
    }

    #[test]
    fn crop_bbox_returns_one_pixel_when_inverted_coordinates() {
        let image = solid_image(100, 100);
        let cropped = crop_bbox(&image, &bbox(30, 30, 10, 10));
        assert_eq!(cropped.width(), 1);
        assert_eq!(cropped.height(), 1);
    }

    #[test]
    fn crop_bbox_returns_one_pixel_when_zero_width() {
        let image = solid_image(100, 100);
        let cropped = crop_bbox(&image, &bbox(20, 10, 20, 40));
        assert_eq!(cropped.width(), 1);
        assert_eq!(cropped.height(), 1);
    }
}
