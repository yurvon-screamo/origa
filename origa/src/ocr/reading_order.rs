use super::deim::BoundingBox;

pub fn sort_reading_order(boxes: &mut [BoundingBox], _img_height: u32, _img_width: u32) {
    if boxes.is_empty() {
        return;
    }

    let vertical_count = boxes
        .iter()
        .filter(|b| (b.y1 - b.y0) > (b.x1 - b.x0))
        .count();

    let is_vertical = vertical_count > boxes.len() / 2;

    if is_vertical {
        boxes.sort_by(|a, b| match b.x0.cmp(&a.x0) {
            std::cmp::Ordering::Equal => a.y0.cmp(&b.y0),
            other => other,
        });
    } else {
        boxes.sort_by(|a, b| match a.y0.cmp(&b.y0) {
            std::cmp::Ordering::Equal => a.x0.cmp(&b.x0),
            other => other,
        });
    }
}
