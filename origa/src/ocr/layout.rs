use candle_core::{DType, Device, IndexOp, Result, Tensor, D};
use candle_nn::{batch_norm, conv2d, conv2d_no_bias, Conv2d, Conv2dConfig, Module, VarBuilder};
use candle_transformers::object_detection::{non_maximum_suppression, Bbox, KeyPoint};
use image::{imageops::FilterType, DynamicImage};
use tracing::{debug, info, span, Level};

use crate::domain::OrigaError;

const CONFIDENCE_THRESHOLD: f32 = 0.25;
const NMS_THRESHOLD: f32 = 0.45;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LayoutClass {
    Caption,
    Footnote,
    Formula,
    ListItem,
    PageFooter,
    PageHeader,
    Picture,
    SectionHeader,
    Table,
    Text,
    Title,
}

impl LayoutClass {
    pub fn is_text_like(&self) -> bool {
        matches!(
            self,
            Self::Text
                | Self::Title
                | Self::SectionHeader
                | Self::Caption
                | Self::Footnote
                | Self::ListItem
        )
    }
}

impl From<usize> for LayoutClass {
    fn from(i: usize) -> Self {
        match i {
            0 => Self::Caption,
            1 => Self::Footnote,
            2 => Self::Formula,
            3 => Self::ListItem,
            4 => Self::PageFooter,
            5 => Self::PageHeader,
            6 => Self::Picture,
            7 => Self::SectionHeader,
            8 => Self::Table,
            9 => Self::Text,
            10 => Self::Title,
            _ => Self::Text,
        }
    }
}

#[derive(Debug, Clone)]
pub struct BoundingBox {
    pub xmin: f32,
    pub ymin: f32,
    pub xmax: f32,
    pub ymax: f32,
    pub class: LayoutClass,
}

pub struct LayoutModel {
    model: YoloV8,
    device: Device,
}

impl LayoutModel {
    pub fn from_bytes(bytes: Vec<u8>) -> std::result::Result<Self, OrigaError> {
        let device = Device::Cpu;
        let multiples = Multiples::x();

        let vb =
            VarBuilder::from_buffered_safetensors(bytes, DType::F32, &device).map_err(|e| {
                OrigaError::OcrError {
                    reason: format!("Failed to load layout model: {}", e),
                }
            })?;

        let model = YoloV8::load(vb, multiples, 11).map_err(|e| OrigaError::OcrError {
            reason: format!("Failed to load YOLOv8 model: {}", e),
        })?;

        Ok(Self { model, device })
    }

    pub fn run_layout_model(
        &self,
        img: &DynamicImage,
    ) -> std::result::Result<Vec<BoundingBox>, OrigaError> {
        info!("Running layout model");
        let (width, height) = {
            let w = img.width() as usize;
            let h = img.height() as usize;
            if w < h {
                let w = w * 640 / h;
                // Sizes have to be divisible by 32.
                (w / 32 * 32, 640)
            } else {
                let h = h * 640 / w;
                (640, h / 32 * 32)
            }
        };

        debug!(width, height, "Resizing image for layout analysis");
        let image_t = {
            let resized = img.resize_exact(width as u32, height as u32, FilterType::CatmullRom);
            let data = resized.to_rgb8().into_raw();
            Tensor::from_vec(
                data,
                (resized.height() as usize, resized.width() as usize, 3),
                &self.device,
            )
            .map_err(|e| OrigaError::OcrError {
                reason: format!("Candle error: {}", e),
            })?
            .permute((2, 0, 1))
            .map_err(|e| OrigaError::OcrError {
                reason: format!("Candle error: {}", e),
            })?
        };

        let image_t = (image_t
            .unsqueeze(0)
            .map_err(|e| OrigaError::OcrError {
                reason: format!("Candle error: {}", e),
            })?
            .to_dtype(DType::F32)
            .map_err(|e| OrigaError::OcrError {
                reason: format!("Candle error: {}", e),
            })?
            * (1. / 255.))
            .map_err(|e| OrigaError::OcrError {
                reason: format!("Candle error: {}", e),
            })?;

        let predictions = self
            .model
            .forward(&image_t)
            .map_err(|e| OrigaError::OcrError {
                reason: format!("Candle error: {}", e),
            })?
            .squeeze(0)
            .map_err(|e| OrigaError::OcrError {
                reason: format!("Candle error: {}", e),
            })?;

        let bboxes = self.process_predictions(&predictions, img, width, height)?;
        Ok(bboxes)
    }

    fn process_predictions(
        &self,
        pred: &Tensor,
        original_img: &DynamicImage,
        w: usize,
        h: usize,
    ) -> std::result::Result<Vec<BoundingBox>, OrigaError> {
        let (dim_size, _num_anchors) = pred.dims2().map_err(|e| OrigaError::OcrError {
            reason: format!("Candle error: {}", e),
        })?;

        let nclasses = dim_size - 4;

        // 1. Отделяем координаты боксов от скоров классов
        let bbox_coords = pred.narrow(0, 0, 4).map_err(|e| OrigaError::OcrError {
            reason: format!("Candle error: {}", e),
        })?;
        let class_scores = pred
            .narrow(0, 4, nclasses)
            .map_err(|e| OrigaError::OcrError {
                reason: format!("Candle error: {}", e),
            })?;

        // 2. Находим максимальный скор для каждого анкора
        let max_scores = class_scores
            .max_keepdim(0)
            .map_err(|e| OrigaError::OcrError {
                reason: format!("Candle error: {}", e),
            })?;

        // 3. Создаем маску: оставляем только те анкоры, где уверенность выше порога
        let mask = max_scores
            .ge(CONFIDENCE_THRESHOLD)
            .map_err(|e| OrigaError::OcrError {
                reason: format!("Candle error: {}", e),
            })?
            .squeeze(0)
            .map_err(|e| OrigaError::OcrError {
                reason: format!("Candle error: {}", e),
            })?;

        // 4. Получаем индексы «выживших» анкоров
        let mask_vec = mask.to_vec1::<u8>().map_err(|e| OrigaError::OcrError {
            reason: format!("Candle error: {}", e),
        })?;

        let mut bboxes: Vec<Vec<Bbox<Vec<KeyPoint>>>> = (0..nclasses).map(|_| vec![]).collect();

        // 5. Итерируемся только по тем индексам, которые прошли порог
        for (idx, &is_candidate) in mask_vec.iter().enumerate() {
            if is_candidate == 0 {
                continue;
            }

            let current_coords: Vec<f32> = bbox_coords
                .i((.., idx))
                .and_then(|t| t.to_vec1())
                .map_err(|e| OrigaError::OcrError {
                    reason: format!("Candle error: {}", e),
                })?;
            let current_class_scores: Vec<f32> = class_scores
                .i((.., idx))
                .and_then(|t| t.to_vec1())
                .map_err(|e| OrigaError::OcrError {
                    reason: format!("Candle error: {}", e),
                })?;

            let (conf, class_idx) = current_class_scores
                .iter()
                .enumerate()
                .max_by(|(_, a), (_, b)| a.total_cmp(b))
                .map(|(i, &s)| (s, i))
                .unwrap();

            let x_center = current_coords[0];
            let y_center = current_coords[1];
            let width = current_coords[2];
            let height = current_coords[3];

            bboxes[class_idx].push(Bbox {
                xmin: x_center - width / 2.,
                ymin: y_center - height / 2.,
                xmax: x_center + width / 2.,
                ymax: y_center + height / 2.,
                confidence: conf,
                data: vec![],
            });
        }

        non_maximum_suppression(&mut bboxes, NMS_THRESHOLD);

        let (initial_w, initial_h) = (original_img.width() as f32, original_img.height() as f32);
        let w_ratio = initial_w / w as f32;
        let h_ratio = initial_h / h as f32;

        let mut result_bboxes = Vec::new();
        for (class_index, bboxes_for_class) in bboxes.iter().enumerate() {
            let class = LayoutClass::from(class_index);
            for b in bboxes_for_class.iter() {
                result_bboxes.push(BoundingBox {
                    xmin: b.xmin * w_ratio,
                    ymin: b.ymin * h_ratio,
                    xmax: b.xmax * w_ratio,
                    ymax: b.ymax * h_ratio,
                    class,
                });
            }
        }

        Ok(result_bboxes)
    }
}

// YOLOv8 Architecture Components

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Multiples {
    depth: f64,
    width: f64,
    ratio: f64,
}

impl Multiples {
    pub fn x() -> Self {
        Self {
            depth: 1.00,
            width: 1.25,
            ratio: 1.0,
        }
    }

    fn filters(&self) -> (usize, usize, usize) {
        let f1 = (256. * self.width) as usize;
        let f2 = (512. * self.width) as usize;
        let f3 = (512. * self.width * self.ratio) as usize;
        (f1, f2, f3)
    }
}

#[derive(Debug)]
struct Upsample {
    scale_factor: usize,
}

impl Upsample {
    fn new(scale_factor: usize) -> Result<Self> {
        Ok(Upsample { scale_factor })
    }
}

impl Module for Upsample {
    fn forward(&self, xs: &Tensor) -> Result<Tensor> {
        let (_b_size, _channels, h, w) = xs.dims4()?;
        xs.upsample_nearest2d(self.scale_factor * h, self.scale_factor * w)
    }
}

#[derive(Debug)]
struct ConvBlock {
    conv: Conv2d,
    span: tracing::Span,
}

impl ConvBlock {
    fn load(
        vb: VarBuilder,
        c1: usize,
        c2: usize,
        k: usize,
        stride: usize,
        padding: Option<usize>,
    ) -> Result<Self> {
        let padding = padding.unwrap_or(k / 2);
        let cfg = Conv2dConfig {
            padding,
            stride,
            groups: 1,
            dilation: 1,
            cudnn_fwd_algo: None,
        };

        let bn = batch_norm(c2, 1e-3, vb.pp("bn"))?;
        let conv = conv2d_no_bias(c1, c2, k, cfg, vb.pp("conv"))?.absorb_bn(&bn)?;
        Ok(Self {
            conv,
            span: span!(Level::TRACE, "conv-block"),
        })
    }
}

impl Module for ConvBlock {
    fn forward(&self, xs: &Tensor) -> Result<Tensor> {
        let _enter = self.span.enter();
        let xs = self.conv.forward(xs)?;
        candle_nn::ops::silu(&xs)
    }
}

#[derive(Debug)]
struct Bottleneck {
    cv1: ConvBlock,
    cv2: ConvBlock,
    residual: bool,
    span: tracing::Span,
}

impl Bottleneck {
    fn load(vb: VarBuilder, c1: usize, c2: usize, shortcut: bool) -> Result<Self> {
        let channel_factor = 1.;
        let c_ = (c2 as f64 * channel_factor) as usize;
        let cv1 = ConvBlock::load(vb.pp("cv1"), c1, c_, 3, 1, None)?;
        let cv2 = ConvBlock::load(vb.pp("cv2"), c_, c2, 3, 1, None)?;
        let residual = c1 == c2 && shortcut;
        Ok(Self {
            cv1,
            cv2,
            residual,
            span: span!(Level::TRACE, "bottleneck"),
        })
    }
}

impl Module for Bottleneck {
    fn forward(&self, xs: &Tensor) -> Result<Tensor> {
        let _enter = self.span.enter();
        let ys = self.cv2.forward(&self.cv1.forward(xs)?)?;
        if self.residual {
            xs + ys
        } else {
            Ok(ys)
        }
    }
}

#[derive(Debug)]
struct C2f {
    cv1: ConvBlock,
    cv2: ConvBlock,
    bottleneck: Vec<Bottleneck>,
    span: tracing::Span,
}

impl C2f {
    fn load(vb: VarBuilder, c1: usize, c2: usize, n: usize, shortcut: bool) -> Result<Self> {
        let c = (c2 as f64 * 0.5) as usize;
        let cv1 = ConvBlock::load(vb.pp("cv1"), c1, 2 * c, 1, 1, None)?;
        let cv2 = ConvBlock::load(vb.pp("cv2"), (2 + n) * c, c2, 1, 1, None)?;
        let mut bottleneck = Vec::with_capacity(n);
        for idx in 0..n {
            let b = Bottleneck::load(vb.pp(format!("bottleneck.{idx}")), c, c, shortcut)?;
            bottleneck.push(b)
        }
        Ok(Self {
            cv1,
            cv2,
            bottleneck,
            span: span!(Level::TRACE, "c2f"),
        })
    }
}

impl Module for C2f {
    fn forward(&self, xs: &Tensor) -> Result<Tensor> {
        let _enter = self.span.enter();
        let ys = self.cv1.forward(xs)?;
        let mut ys_chunks = ys.chunk(2, 1)?;
        for m in self.bottleneck.iter() {
            ys_chunks.push(m.forward(ys_chunks.last().unwrap())?)
        }
        let zs = Tensor::cat(ys_chunks.as_slice(), 1)?;
        self.cv2.forward(&zs)
    }
}

#[derive(Debug)]
struct Sppf {
    cv1: ConvBlock,
    cv2: ConvBlock,
    k: usize,
    span: tracing::Span,
}

impl Sppf {
    fn load(vb: VarBuilder, c1: usize, c2: usize, k: usize) -> Result<Self> {
        let c_ = c1 / 2;
        let cv1 = ConvBlock::load(vb.pp("cv1"), c1, c_, 1, 1, None)?;
        let cv2 = ConvBlock::load(vb.pp("cv2"), c_ * 4, c2, 1, 1, None)?;
        Ok(Self {
            cv1,
            cv2,
            k,
            span: span!(Level::TRACE, "sppf"),
        })
    }
}

impl Module for Sppf {
    fn forward(&self, xs: &Tensor) -> Result<Tensor> {
        let _enter = self.span.enter();
        let xs = self.cv1.forward(xs)?;
        let xs2 = xs
            .pad_with_zeros(2, self.k / 2, self.k / 2)?
            .pad_with_zeros(3, self.k / 2, self.k / 2)?
            .max_pool2d_with_stride(self.k, 1)?;
        let xs3 = xs2
            .pad_with_zeros(2, self.k / 2, self.k / 2)?
            .pad_with_zeros(3, self.k / 2, self.k / 2)?
            .max_pool2d_with_stride(self.k, 1)?;
        let xs4 = xs3
            .pad_with_zeros(2, self.k / 2, self.k / 2)?
            .pad_with_zeros(3, self.k / 2, self.k / 2)?
            .max_pool2d_with_stride(self.k, 1)?;
        self.cv2.forward(&Tensor::cat(&[&xs, &xs2, &xs3, &xs4], 1)?)
    }
}

#[derive(Debug)]
struct Dfl {
    conv: Conv2d,
    num_classes: usize,
    span: tracing::Span,
}

impl Dfl {
    fn load(vb: VarBuilder, num_classes: usize) -> Result<Self> {
        let conv = conv2d_no_bias(num_classes, 1, 1, Default::default(), vb.pp("conv"))?;
        Ok(Self {
            conv,
            num_classes,
            span: span!(Level::TRACE, "dfl"),
        })
    }
}

impl Module for Dfl {
    fn forward(&self, xs: &Tensor) -> Result<Tensor> {
        let _enter = self.span.enter();
        let (b_sz, _channels, anchors) = xs.dims3()?;
        let xs = xs
            .reshape((b_sz, 4, self.num_classes, anchors))?
            .transpose(2, 1)?;
        let xs = candle_nn::ops::softmax(&xs, 1)?;
        self.conv.forward(&xs)?.reshape((b_sz, 4, anchors))
    }
}

#[derive(Debug)]
struct DarkNet {
    b1_0: ConvBlock,
    b1_1: ConvBlock,
    b2_0: C2f,
    b2_1: ConvBlock,
    b2_2: C2f,
    b3_0: ConvBlock,
    b3_1: C2f,
    b4_0: ConvBlock,
    b4_1: C2f,
    b5: Sppf,
    span: tracing::Span,
}

impl DarkNet {
    fn load(vb: VarBuilder, m: Multiples) -> Result<Self> {
        let (w, r, d) = (m.width, m.ratio, m.depth);
        let b1_0 = ConvBlock::load(vb.pp("b1.0"), 3, (64. * w) as usize, 3, 2, Some(1))?;
        let b1_1 = ConvBlock::load(
            vb.pp("b1.1"),
            (64. * w) as usize,
            (128. * w) as usize,
            3,
            2,
            Some(1),
        )?;
        let b2_0 = C2f::load(
            vb.pp("b2.0"),
            (128. * w) as usize,
            (128. * w) as usize,
            (3. * d).round() as usize,
            true,
        )?;
        let b2_1 = ConvBlock::load(
            vb.pp("b2.1"),
            (128. * w) as usize,
            (256. * w) as usize,
            3,
            2,
            Some(1),
        )?;
        let b2_2 = C2f::load(
            vb.pp("b2.2"),
            (256. * w) as usize,
            (256. * w) as usize,
            (6. * d).round() as usize,
            true,
        )?;
        let b3_0 = ConvBlock::load(
            vb.pp("b3.0"),
            (256. * w) as usize,
            (512. * w) as usize,
            3,
            2,
            Some(1),
        )?;
        let b3_1 = C2f::load(
            vb.pp("b3.1"),
            (512. * w) as usize,
            (512. * w) as usize,
            (6. * d).round() as usize,
            true,
        )?;
        let b4_0 = ConvBlock::load(
            vb.pp("b4.0"),
            (512. * w) as usize,
            (512. * w * r) as usize,
            3,
            2,
            Some(1),
        )?;
        let b4_1 = C2f::load(
            vb.pp("b4.1"),
            (512. * w * r) as usize,
            (512. * w * r) as usize,
            (3. * d).round() as usize,
            true,
        )?;
        let b5 = Sppf::load(
            vb.pp("b5.0"),
            (512. * w * r) as usize,
            (512. * w * r) as usize,
            5,
        )?;
        Ok(Self {
            b1_0,
            b1_1,
            b2_0,
            b2_1,
            b2_2,
            b3_0,
            b3_1,
            b4_0,
            b4_1,
            b5,
            span: span!(Level::TRACE, "darknet"),
        })
    }

    fn forward(&self, xs: &Tensor) -> Result<(Tensor, Tensor, Tensor)> {
        let _enter = self.span.enter();
        let x1 = self.b1_1.forward(&self.b1_0.forward(xs)?)?;
        let x2 = self
            .b2_2
            .forward(&self.b2_1.forward(&self.b2_0.forward(&x1)?)?)?;
        let x3 = self.b3_1.forward(&self.b3_0.forward(&x2)?)?;
        let x4 = self.b4_1.forward(&self.b4_0.forward(&x3)?)?;
        let x5 = self.b5.forward(&x4)?;
        Ok((x2, x3, x5))
    }
}

#[derive(Debug)]
struct YoloV8Neck {
    up: Upsample,
    n1: C2f,
    n2: C2f,
    n3: ConvBlock,
    n4: C2f,
    n5: ConvBlock,
    n6: C2f,
    span: tracing::Span,
}

impl YoloV8Neck {
    fn load(vb: VarBuilder, m: Multiples) -> Result<Self> {
        let up = Upsample::new(2)?;
        let (w, r, d) = (m.width, m.ratio, m.depth);
        let n = (3. * d).round() as usize;
        let n1 = C2f::load(
            vb.pp("n1"),
            (512. * w * (1. + r)) as usize,
            (512. * w) as usize,
            n,
            false,
        )?;
        let n2 = C2f::load(
            vb.pp("n2"),
            (768. * w) as usize,
            (256. * w) as usize,
            n,
            false,
        )?;
        let n3 = ConvBlock::load(
            vb.pp("n3"),
            (256. * w) as usize,
            (256. * w) as usize,
            3,
            2,
            Some(1),
        )?;
        let n4 = C2f::load(
            vb.pp("n4"),
            (768. * w) as usize,
            (512. * w) as usize,
            n,
            false,
        )?;
        let n5 = ConvBlock::load(
            vb.pp("n5"),
            (512. * w) as usize,
            (512. * w) as usize,
            3,
            2,
            Some(1),
        )?;
        let n6 = C2f::load(
            vb.pp("n6"),
            (512. * w * (1. + r)) as usize,
            (512. * w * r) as usize,
            n,
            false,
        )?;
        Ok(Self {
            up,
            n1,
            n2,
            n3,
            n4,
            n5,
            n6,
            span: span!(Level::TRACE, "neck"),
        })
    }

    fn forward(&self, p3: &Tensor, p4: &Tensor, p5: &Tensor) -> Result<(Tensor, Tensor, Tensor)> {
        let _enter = self.span.enter();
        let x = self
            .n1
            .forward(&Tensor::cat(&[&self.up.forward(p5)?, p4], 1)?)?;
        let head_1 = self
            .n2
            .forward(&Tensor::cat(&[&self.up.forward(&x)?, p3], 1)?)?;
        let head_2 = self
            .n4
            .forward(&Tensor::cat(&[&self.n3.forward(&head_1)?, &x], 1)?)?;
        let head_3 = self
            .n6
            .forward(&Tensor::cat(&[&self.n5.forward(&head_2)?, p5], 1)?)?;
        Ok((head_1, head_2, head_3))
    }
}

#[derive(Debug)]
struct DetectionHead {
    dfl: Dfl,
    cv2: [(ConvBlock, ConvBlock, Conv2d); 3],
    cv3: [(ConvBlock, ConvBlock, Conv2d); 3],
    ch: usize,
    no: usize,
    span: tracing::Span,
}

fn make_anchors(
    xs0: &Tensor,
    xs1: &Tensor,
    xs2: &Tensor,
    (s0, s1, s2): (usize, usize, usize),
    grid_cell_offset: f64,
) -> Result<(Tensor, Tensor)> {
    let dev = xs0.device();
    let mut anchor_points = vec![];
    let mut stride_tensor = vec![];
    for (xs, stride) in [(xs0, s0), (xs1, s1), (xs2, s2)] {
        let (_, _, h, w) = xs.dims4()?;
        let sx = (Tensor::arange(0, w as u32, dev)?.to_dtype(DType::F32)? + grid_cell_offset)?;
        let sy = (Tensor::arange(0, h as u32, dev)?.to_dtype(DType::F32)? + grid_cell_offset)?;
        let sx = sx
            .reshape((1, sx.elem_count()))?
            .repeat((h, 1))?
            .flatten_all()?;
        let sy = sy
            .reshape((sy.elem_count(), 1))?
            .repeat((1, w))?
            .flatten_all()?;
        anchor_points.push(Tensor::stack(&[&sx, &sy], D::Minus1)?);
        stride_tensor.push((Tensor::ones(h * w, DType::F32, dev)? * stride as f64)?);
    }
    let anchor_points = Tensor::cat(anchor_points.as_slice(), 0)?;
    let stride_tensor = Tensor::cat(stride_tensor.as_slice(), 0)?.unsqueeze(1)?;
    Ok((anchor_points, stride_tensor))
}

fn dist2bbox(distance: &Tensor, anchor_points: &Tensor) -> Result<Tensor> {
    let chunks = distance.chunk(2, 1)?;
    let lt = &chunks[0];
    let rb = &chunks[1];
    let x1y1 = anchor_points.sub(lt)?;
    let x2y2 = anchor_points.add(rb)?;
    let c_xy = ((&x1y1 + &x2y2)? * 0.5)?;
    let wh = (&x2y2 - &x1y1)?;
    Tensor::cat(&[c_xy, wh], 1)
}

struct DetectionHeadOut {
    pred: Tensor,
}

impl DetectionHead {
    fn load(vb: VarBuilder, nc: usize, filters: (usize, usize, usize)) -> Result<Self> {
        let ch = 16;
        let dfl = Dfl::load(vb.pp("dfl"), ch)?;
        let c1 = usize::max(filters.0, nc);
        let c2 = usize::max(filters.0 / 4, ch * 4);
        let cv3 = [
            Self::load_cv3(vb.pp("cv3.0"), c1, nc, filters.0)?,
            Self::load_cv3(vb.pp("cv3.1"), c1, nc, filters.1)?,
            Self::load_cv3(vb.pp("cv3.2"), c1, nc, filters.2)?,
        ];
        let cv2 = [
            Self::load_cv2(vb.pp("cv2.0"), c2, ch, filters.0)?,
            Self::load_cv2(vb.pp("cv2.1"), c2, ch, filters.1)?,
            Self::load_cv2(vb.pp("cv2.2"), c2, ch, filters.2)?,
        ];
        let no = nc + ch * 4;
        Ok(Self {
            dfl,
            cv2,
            cv3,
            ch,
            no,
            span: span!(Level::TRACE, "detection-head"),
        })
    }

    fn load_cv3(
        vb: VarBuilder,
        c1: usize,
        nc: usize,
        filter: usize,
    ) -> Result<(ConvBlock, ConvBlock, Conv2d)> {
        let block0 = ConvBlock::load(vb.pp("0"), filter, c1, 3, 1, None)?;
        let block1 = ConvBlock::load(vb.pp("1"), c1, c1, 3, 1, None)?;
        let conv = conv2d(c1, nc, 1, Default::default(), vb.pp("2"))?;
        Ok((block0, block1, conv))
    }

    fn load_cv2(
        vb: VarBuilder,
        c2: usize,
        ch: usize,
        filter: usize,
    ) -> Result<(ConvBlock, ConvBlock, Conv2d)> {
        let block0 = ConvBlock::load(vb.pp("0"), filter, c2, 3, 1, None)?;
        let block1 = ConvBlock::load(vb.pp("1"), c2, c2, 3, 1, None)?;
        let conv = conv2d(c2, 4 * ch, 1, Default::default(), vb.pp("2"))?;
        Ok((block0, block1, conv))
    }

    fn forward(&self, xs0: &Tensor, xs1: &Tensor, xs2: &Tensor) -> Result<DetectionHeadOut> {
        let _enter = self.span.enter();
        let forward_cv = |xs, i: usize| {
            let xs_2 = self.cv2[i].0.forward(xs)?;
            let xs_2 = self.cv2[i].1.forward(&xs_2)?;
            let xs_2 = self.cv2[i].2.forward(&xs_2)?;

            let xs_3 = self.cv3[i].0.forward(xs)?;
            let xs_3 = self.cv3[i].1.forward(&xs_3)?;
            let xs_3 = self.cv3[i].2.forward(&xs_3)?;
            Tensor::cat(&[&xs_2, &xs_3], 1)
        };
        let xs0 = forward_cv(xs0, 0)?;
        let xs1 = forward_cv(xs1, 1)?;
        let xs2 = forward_cv(xs2, 2)?;

        let (anchors, strides) = make_anchors(&xs0, &xs1, &xs2, (8, 16, 32), 0.5)?;
        let anchors = anchors.transpose(0, 1)?.unsqueeze(0)?;
        let strides = strides.transpose(0, 1)?;

        let reshape = |xs: &Tensor| {
            let d = xs.dim(0)?;
            let el = xs.elem_count();
            xs.reshape((d, self.no, el / (d * self.no)))
        };
        let ys0 = reshape(&xs0)?;
        let ys1 = reshape(&xs1)?;
        let ys2 = reshape(&xs2)?;

        let x_cat = Tensor::cat(&[ys0, ys1, ys2], 2)?;
        let box_ = x_cat.i((.., ..self.ch * 4))?;
        let cls = x_cat.i((.., self.ch * 4..))?;

        let dbox = dist2bbox(&self.dfl.forward(&box_)?, &anchors)?;
        let dbox = dbox.broadcast_mul(&strides)?;
        let pred = Tensor::cat(&[dbox, candle_nn::ops::sigmoid(&cls)?], 1)?;

        Ok(DetectionHeadOut { pred })
    }
}

#[derive(Debug)]
pub struct YoloV8 {
    net: DarkNet,
    fpn: YoloV8Neck,
    head: DetectionHead,
    span: tracing::Span,
}

impl YoloV8 {
    pub fn load(vb: VarBuilder, m: Multiples, num_classes: usize) -> Result<Self> {
        let net = DarkNet::load(vb.pp("net"), m)?;
        let fpn = YoloV8Neck::load(vb.pp("fpn"), m)?;
        let head = DetectionHead::load(vb.pp("head"), num_classes, m.filters())?;
        Ok(Self {
            net,
            fpn,
            head,
            span: span!(Level::TRACE, "yolo-v8"),
        })
    }
}

impl Module for YoloV8 {
    fn forward(&self, xs: &Tensor) -> Result<Tensor> {
        let _enter = self.span.enter();
        let (xs1, xs2, xs3) = self.net.forward(xs)?;
        let (xs1, xs2, xs3) = self.fpn.forward(&xs1, &xs2, &xs3)?;
        Ok(self.head.forward(&xs1, &xs2, &xs3)?.pred)
    }
}
