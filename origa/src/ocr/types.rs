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
