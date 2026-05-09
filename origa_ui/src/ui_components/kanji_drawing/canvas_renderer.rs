use super::svg_parser::{PathCommand, parse_svg_path_commands};
use super::{HINT_LINE_WIDTH, STROKE_LINE_WIDTH, SVG_SCALE};
use js_sys::Array;
use wasm_bindgen::JsValue;
use web_sys::CanvasRenderingContext2d;

fn get_css_color(var_name: &str) -> String {
    let window = match web_sys::window() {
        Some(w) => w,
        None => return "#000000".to_string(),
    };
    let document = match window.document() {
        Some(d) => d,
        None => return "#000000".to_string(),
    };
    let root = match document.document_element() {
        Some(e) => e,
        None => return "#000000".to_string(),
    };
    let style = match window.get_computed_style(&root) {
        Ok(Some(s)) => s,
        _ => return "#000000".to_string(),
    };
    style
        .get_property_value(var_name)
        .unwrap_or_default()
        .trim()
        .to_string()
}

pub fn get_stroke_color() -> String {
    get_css_color("--fg-black")
}

pub fn get_hint_color() -> String {
    get_css_color("--accent-terracotta")
}

pub fn get_user_color() -> String {
    get_css_color("--accent-olive")
}

pub fn redraw_canvas(
    ctx: &CanvasRenderingContext2d,
    strokes: &[super::StrokeData],
    current_index: usize,
    canvas_size: u32,
) {
    ctx.clear_rect(0.0, 0.0, canvas_size as f64, canvas_size as f64);
    for (i, stroke) in strokes.iter().enumerate() {
        if i < current_index {
            draw_completed_stroke(ctx, &stroke.d);
        } else if i == current_index {
            draw_hint_stroke(ctx, &stroke.d);
        }
    }
}

fn draw_completed_stroke(ctx: &CanvasRenderingContext2d, d: &str) {
    ctx.set_line_width(STROKE_LINE_WIDTH);
    ctx.set_stroke_style_str(&get_stroke_color());
    ctx.set_line_cap("round");
    ctx.set_line_join("round");
    ctx.set_line_dash(&js_sys::Array::new()).ok();
    draw_path_from_d(ctx, d);
}

fn draw_hint_stroke(ctx: &CanvasRenderingContext2d, d: &str) {
    ctx.set_line_width(HINT_LINE_WIDTH);
    ctx.set_stroke_style_str(&get_hint_color());
    ctx.set_line_cap("round");
    ctx.set_line_join("round");
    let dash_array = Array::new_with_length(2);
    dash_array.set(0, JsValue::from_f64(10.0));
    dash_array.set(1, JsValue::from_f64(5.0));
    ctx.set_line_dash(&dash_array).ok();
    draw_path_from_d(ctx, d);
    ctx.set_line_dash(&Array::new()).ok();
}

fn draw_path_from_d(ctx: &CanvasRenderingContext2d, d: &str) {
    ctx.begin_path();
    let commands = parse_svg_path_commands(d);
    for cmd in &commands {
        match cmd {
            PathCommand::MoveTo(x, y) => {
                ctx.move_to(x * SVG_SCALE, y * SVG_SCALE);
            },
            PathCommand::LineTo(x, y) => {
                ctx.line_to(x * SVG_SCALE, y * SVG_SCALE);
            },
            PathCommand::CurveTo(x1, y1, x2, y2, x, y) => {
                ctx.bezier_curve_to(
                    x1 * SVG_SCALE,
                    y1 * SVG_SCALE,
                    x2 * SVG_SCALE,
                    y2 * SVG_SCALE,
                    x * SVG_SCALE,
                    y * SVG_SCALE,
                );
            },
            PathCommand::ClosePath => {
                ctx.close_path();
            },
        }
    }
    ctx.stroke();
}
