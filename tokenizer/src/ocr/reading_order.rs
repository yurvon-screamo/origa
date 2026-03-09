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
        boxes.sort_by(|a, b| {
            let col_a = a.x0 / 50;
            let col_b = b.x0 / 50;
            if col_a != col_b {
                col_b.cmp(&col_a)
            } else {
                a.y0.cmp(&b.y0)
            }
        });
    } else {
        boxes.sort_by(|a, b| {
            let row_a = a.y0 / 20;
            let row_b = b.y0 / 20;
            if row_a != row_b {
                row_a.cmp(&row_b)
            } else {
                a.x0.cmp(&b.x0)
            }
        });
    }
}
