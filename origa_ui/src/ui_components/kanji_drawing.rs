use crate::core::config::public_url;
use leptos::ev::PointerEvent;
use leptos::html::Canvas;
use leptos::prelude::*;
use leptos::wasm_bindgen::JsCast;
use std::sync::{Arc, Mutex};
use tracing::debug;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use web_sys::CanvasRenderingContext2d;
use web_sys::js_sys::Array;

const CANVAS_SIZE: u32 = 320;
const SVG_VIEWBOX_SIZE: f64 = 109.0;
const SVG_SCALE: f64 = CANVAS_SIZE as f64 / SVG_VIEWBOX_SIZE;
const CANVAS_TOLERANCE: f64 = 25.0; // Max distance in canvas pixels (~8% of 320px)
const SUCCESS_THRESHOLD: f64 = 0.80; // 80% of reference points must be covered
const SAMPLE_COUNT: usize = 21; // Number of points to sample from reference stroke

const MIN_STROKE_POINTS: usize = 8;
const STROKE_LINE_WIDTH: f64 = 12.0;
const HINT_LINE_WIDTH: f64 = 4.0;
const USER_LINE_WIDTH: f64 = 8.0;

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

fn get_stroke_color() -> String {
    get_css_color("--fg-black")
}

fn get_hint_color() -> String {
    get_css_color("--accent-terracotta")
}

fn get_user_color() -> String {
    get_css_color("--accent-olive")
}

#[derive(Clone, Default)]
struct StrokeData {
    d: String,
}

#[derive(Clone, Default)]
struct DrawingState {
    points: Vec<(f64, f64)>,
    is_drawing: bool,
}

type CanvasContext = Arc<Mutex<Option<CanvasRenderingContext2d>>>;
type DrawingStateRef = Arc<Mutex<DrawingState>>;

#[component]
pub fn KanjiDrawingPractice(
    kanji: String,
    #[prop(optional)] on_complete: Option<Callback<()>>,
) -> impl IntoView {
    let svg_content = LocalResource::new(move || {
        let encoded = urlencoding::encode(&kanji);
        let path = public_url(&format!("/public/kanji_animations/{}.svg", encoded));
        async move { fetch_svg(&path).await }
    });

    let strokes = RwSignal::new(Vec::<StrokeData>::new());
    let current_stroke_index = RwSignal::new(0usize);
    let is_completed = RwSignal::new(false);
    let load_error = RwSignal::new(false);
    let canvas_ref: NodeRef<Canvas> = NodeRef::new();
    let drawing_state: DrawingStateRef = Arc::new(Mutex::new(DrawingState::default()));
    let ctx_storage: CanvasContext = Arc::new(Mutex::new(None));
    let ctx_storage_clone = ctx_storage.clone();
    Effect::new(move |_| match svg_content.get() {
        Some(Some(svg)) => {
            let parsed = parse_stroke_paths(&svg);
            if parsed.is_empty() {
                load_error.set(true);
            } else {
                strokes.set(parsed);
                current_stroke_index.set(0);
                is_completed.set(false);
                load_error.set(false);
            }
        }
        Some(None) => {
            load_error.set(true);
        }
        None => {}
    });
    Effect::new(move |_| {
        let canvas = canvas_ref.get()?;
        let canvas: web_sys::HtmlCanvasElement = canvas.unchecked_into();
        let ctx = canvas
            .get_context("2d")
            .ok()?
            .and_then(|v| v.dyn_into::<CanvasRenderingContext2d>().ok())?;
        redraw_canvas(&ctx, &strokes.get(), current_stroke_index.get());
        ctx_storage_clone.lock().ok()?.replace(ctx);
        Some(())
    });
    let ctx_storage_for_update = ctx_storage.clone();
    Effect::new(move |_| {
        let ctx_guard = ctx_storage_for_update.lock().ok()?;
        let Some(ctx) = ctx_guard.as_ref() else {
            drop(ctx_guard);
            return Some(());
        };
        let stroke_idx = current_stroke_index.get();
        let stroke_list = strokes.get();
        let completed = is_completed.get();

        // Always redraw, even when completed - this ensures last stroke is shown
        redraw_canvas(ctx, &stroke_list, stroke_idx);

        if completed {
            return Some(());
        }
        Some(())
    });
    let handle_pointer_down = {
        let state = drawing_state.clone();
        let ctx_store = ctx_storage.clone();
        move |ev: PointerEvent| {
            let canvas = canvas_ref.get();
            let Some(canvas) = canvas else { return };
            let canvas: web_sys::HtmlCanvasElement = canvas.unchecked_into();
            let ctx_guard = ctx_store.lock().ok();
            let Some(ref ctx_guard) = ctx_guard else {
                return;
            };
            let Some(ctx) = ctx_guard.as_ref() else {
                return;
            };
            let rect = canvas.get_bounding_client_rect();
            let x = ev.client_x() as f64 - rect.left();
            let y = ev.client_y() as f64 - rect.top();
            let mut state = state.lock().ok();
            let Some(ref mut state) = state else { return };
            state.points = vec![(x, y)];
            state.is_drawing = true;
            ctx.set_line_width(USER_LINE_WIDTH);
            ctx.set_stroke_style_str(&get_user_color());
            ctx.set_line_cap("round");
            ctx.set_line_join("round");
            ctx.begin_path();
            ctx.move_to(x, y);
        }
    };
    let handle_pointer_move = {
        let state = drawing_state.clone();
        let ctx_store = ctx_storage.clone();
        move |ev: PointerEvent| {
            {
                let state_guard = state.lock().ok();
                let Some(ref state_guard) = state_guard else {
                    return;
                };
                if !state_guard.is_drawing {
                    return;
                }
            }
            let canvas = canvas_ref.get();
            let Some(canvas) = canvas else { return };
            let canvas: web_sys::HtmlCanvasElement = canvas.unchecked_into();
            let ctx_guard = ctx_store.lock().ok();
            let Some(ref ctx_guard) = ctx_guard else {
                return;
            };
            let Some(ctx) = ctx_guard.as_ref() else {
                return;
            };
            let rect = canvas.get_bounding_client_rect();
            let x = ev.client_x() as f64 - rect.left();
            let y = ev.client_y() as f64 - rect.top();
            {
                let mut state_guard = state.lock().ok();
                let Some(ref mut state_guard) = state_guard else {
                    return;
                };
                state_guard.points.push((x, y));
            }
            ctx.line_to(x, y);
            ctx.stroke();
            ctx.begin_path();
            ctx.move_to(x, y);
        }
    };
    let handle_pointer_up = {
        let state = drawing_state.clone();
        let ctx_store = ctx_storage.clone();
        move |_| {
            let points: Vec<(f64, f64)> = {
                let mut state_guard = state.lock().ok();
                let Some(ref mut state_guard) = state_guard else {
                    return;
                };
                state_guard.is_drawing = false;
                let pts = state_guard.points.clone();
                state_guard.points.clear();
                pts
            };
            if points.len() < MIN_STROKE_POINTS {
                current_stroke_index.update(|_| {});
                return;
            }
            let stroke_list = strokes.get();
            let current_idx = current_stroke_index.get();
            if current_idx >= stroke_list.len() {
                return;
            }
            let current_stroke = &stroke_list[current_idx];
            if is_stroke_similar(&points, &current_stroke.d) {
                let next_idx = current_idx + 1;
                if next_idx >= stroke_list.len() {
                    current_stroke_index.set(next_idx);
                    is_completed.set(true);
                    if let Some(cb) = on_complete {
                        cb.run(());
                    }
                } else {
                    current_stroke_index.set(next_idx);
                }
            } else if let Ok(ctx_guard) = ctx_store.lock()
                && let Some(ctx) = ctx_guard.as_ref()
            {
                redraw_canvas(ctx, &strokes.get(), current_stroke_index.get());
            }
        }
    };
    let handle_pointer_leave = handle_pointer_up.clone();
    let reset_practice = {
        let state = drawing_state.clone();
        let ctx_store = ctx_storage.clone();
        move |_| {
            current_stroke_index.set(0);
            is_completed.set(false);
            if let Ok(mut state_guard) = state.lock() {
                state_guard.points.clear();
            }
            if let Ok(ctx_guard) = ctx_store.lock()
                && let Some(ctx) = ctx_guard.as_ref()
            {
                redraw_canvas(ctx, &strokes.get(), 0);
            }
        }
    };
    view! {
        <div class="kanji-drawing-container">
            <div class="kanji-drawing-info">
                {move || {
                    if load_error.get() {
                        view! {
                            <div class="kanji-drawing-error">
                                "Анимация для этого кандзи недоступна"
                            </div>
                        }
                        .into_any()
                    } else if is_completed.get() {
                        view! {
                            <div class="kanji-drawing-success">"Готово!"</div>
                        }
                        .into_any()
                    } else {
                        let total = strokes.get().len();
                        let current = current_stroke_index.get() + 1;
                        view! {
                            <div class="kanji-drawing-progress">
                                {format!("Штрих {} / {}", current, total)}
                            </div>
                        }
                        .into_any()
                    }
                }}
            </div>
            <div class="kanji-drawing-canvas-wrapper">
                <canvas
                    node_ref={canvas_ref}
                    width={CANVAS_SIZE}
                    height={CANVAS_SIZE}
                    class="kanji-drawing-canvas"
                    on:pointerdown={handle_pointer_down}
                    on:pointermove={handle_pointer_move}
                    on:pointerup={handle_pointer_up}
                    on:pointerleave={handle_pointer_leave}
                />
            </div>
            <div class="kanji-drawing-controls">
                <button class="kanji-drawing-reset-btn" on:click={reset_practice}>
                    "Начать заново"
                </button>
            </div>
        </div>
    }
}
async fn fetch_svg(path: &str) -> Option<String> {
    let window = web_sys::window()?;
    let resp = JsFuture::from(window.fetch_with_str(path)).await.ok()?;
    let resp = resp.dyn_into::<web_sys::Response>().ok()?;
    let text = JsFuture::from(resp.text().ok()?).await.ok()?;
    text.as_string()
}
fn parse_stroke_paths(svg: &str) -> Vec<StrokeData> {
    let mut strokes = Vec::new();
    let mut pos = 0;
    while let Some(rel_start) = svg[pos..].find("<path") {
        let abs_start = pos + rel_start;
        let rest = &svg[abs_start..];
        let tag_end = rest.find('>').unwrap_or(rest.len());
        let path_tag = &rest[..=tag_end.min(rest.len() - 1)];
        if !path_tag.contains("class=\"bg\"")
            && !path_tag.contains("class='bg'")
            && let Some(d) = extract_attribute(path_tag, "d")
        {
            debug!(stroke_index = strokes.len(), d = %d, "Parsed stroke path");
            strokes.push(StrokeData { d });
        }
        pos = abs_start + tag_end + 1;
    }
    debug!(
        total_strokes = strokes.len(),
        "Finished parsing SVG strokes"
    );
    strokes
}
fn extract_attribute(tag: &str, attr: &str) -> Option<String> {
    let patterns = [format!("{}=\"", attr), format!("{}='", attr)];
    for pattern in patterns {
        if let Some(start) = tag.find(&pattern) {
            let value_start = start + pattern.len();
            let quote = &pattern[pattern.len() - 1..pattern.len()];
            if let Some(end) = tag[value_start..].find(quote) {
                return Some(tag[value_start..value_start + end].to_string());
            }
        }
    }
    None
}
fn redraw_canvas(ctx: &CanvasRenderingContext2d, strokes: &[StrokeData], current_index: usize) {
    ctx.clear_rect(0.0, 0.0, CANVAS_SIZE as f64, CANVAS_SIZE as f64);
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
    parse_and_draw_svg_path(ctx, d);
    ctx.stroke();
}
fn parse_and_draw_svg_path(ctx: &CanvasRenderingContext2d, d: &str) {
    let chars: Vec<char> = d.chars().collect();
    let mut pos = 0;
    let mut current_cmd = 'M';
    let mut current_pos = (0.0, 0.0);
    while pos < chars.len() {
        let c = chars[pos];
        if c.is_ascii_alphabetic() {
            current_cmd = c;
            pos += 1;
        }
        match current_cmd {
            'M' | 'm' => {
                if let Some((x, y, new_pos)) = parse_coords(&chars, pos) {
                    let (abs_x, abs_y) = if current_cmd == 'm' {
                        (current_pos.0 + x, current_pos.1 + y)
                    } else {
                        (x, y)
                    };
                    ctx.move_to(abs_x * SVG_SCALE, abs_y * SVG_SCALE);
                    current_pos = (abs_x, abs_y);
                    pos = new_pos;
                    current_cmd = if current_cmd == 'm' { 'l' } else { 'L' };
                } else {
                    break;
                }
            }
            'L' | 'l' => {
                if let Some((x, y, new_pos)) = parse_coords(&chars, pos) {
                    let (abs_x, abs_y) = if current_cmd == 'l' {
                        (current_pos.0 + x, current_pos.1 + y)
                    } else {
                        (x, y)
                    };
                    ctx.line_to(abs_x * SVG_SCALE, abs_y * SVG_SCALE);
                    current_pos = (abs_x, abs_y);
                    pos = new_pos;
                } else {
                    break;
                }
            }
            'C' | 'c' => {
                if let Some((x1, y1, x2, y2, x, y, new_pos)) = parse_curve_coords(&chars, pos) {
                    let (abs_x1, abs_y1, abs_x2, abs_y2, abs_x, abs_y) = if current_cmd == 'c' {
                        (
                            current_pos.0 + x1,
                            current_pos.1 + y1,
                            current_pos.0 + x2,
                            current_pos.1 + y2,
                            current_pos.0 + x,
                            current_pos.1 + y,
                        )
                    } else {
                        (x1, y1, x2, y2, x, y)
                    };
                    ctx.bezier_curve_to(
                        abs_x1 * SVG_SCALE,
                        abs_y1 * SVG_SCALE,
                        abs_x2 * SVG_SCALE,
                        abs_y2 * SVG_SCALE,
                        abs_x * SVG_SCALE,
                        abs_y * SVG_SCALE,
                    );
                    current_pos = (abs_x, abs_y);
                    pos = new_pos;
                } else {
                    break;
                }
            }
            'Z' | 'z' => {
                ctx.close_path();
                pos += 1;
            }
            _ => {
                pos += 1;
            }
        }
    }
}
fn parse_coords(chars: &[char], start: usize) -> Option<(f64, f64, usize)> {
    let mut pos = skip_whitespace(chars, start);
    let (x, new_pos) = parse_number(chars, pos)?;
    pos = skip_whitespace_or_comma(chars, new_pos);
    let (y, new_pos) = parse_number(chars, pos)?;
    Some((x, y, new_pos))
}
fn parse_curve_coords(
    chars: &[char],
    start: usize,
) -> Option<(f64, f64, f64, f64, f64, f64, usize)> {
    let (x1, pos) = parse_number(chars, skip_whitespace(chars, start))?;
    let (y1, pos) = parse_number(chars, skip_whitespace_or_comma(chars, pos))?;
    let (x2, pos) = parse_number(chars, skip_whitespace_or_comma(chars, pos))?;
    let (y2, pos) = parse_number(chars, skip_whitespace_or_comma(chars, pos))?;
    let (x, pos) = parse_number(chars, skip_whitespace_or_comma(chars, pos))?;
    let (y, pos) = parse_number(chars, skip_whitespace_or_comma(chars, pos))?;
    Some((x1, y1, x2, y2, x, y, pos))
}
fn skip_whitespace(chars: &[char], start: usize) -> usize {
    let mut pos = start;
    while pos < chars.len() && (chars[pos].is_whitespace() || chars[pos] == ',') {
        pos += 1;
    }
    pos
}
fn skip_whitespace_or_comma(chars: &[char], start: usize) -> usize {
    skip_whitespace(chars, start)
}
fn parse_number(chars: &[char], start: usize) -> Option<(f64, usize)> {
    let mut pos = start;
    let mut num_str = String::new();
    if pos < chars.len() && (chars[pos] == '-' || chars[pos] == '+') {
        num_str.push(chars[pos]);
        pos += 1;
    }
    while pos < chars.len() && (chars[pos].is_ascii_digit() || chars[pos] == '.') {
        num_str.push(chars[pos]);
        pos += 1;
    }
    let num: f64 = num_str.parse().ok()?;
    Some((num, pos))
}

fn is_stroke_similar(user_points: &[(f64, f64)], stroke_d: &str) -> bool {
    debug!("[Stroke Check] user_points count: {}", user_points.len());

    if user_points.len() < MIN_STROKE_POINTS {
        debug!(
            "[Stroke Check] FAIL: too few points ({})",
            user_points.len()
        );
        return false;
    }

    let stroke_samples = sample_stroke_path(stroke_d);
    debug!(
        "[Stroke Check] stroke_samples count: {}",
        stroke_samples.len()
    );

    if stroke_samples.len() < SAMPLE_COUNT {
        debug!("[Stroke Check] FAIL: too few stroke samples");
        return false;
    }

    // Debug: show raw coordinates range
    if !user_points.is_empty() {
        let (min_x, max_x, min_y, max_y) = user_points.iter().fold(
            (
                f64::INFINITY,
                f64::NEG_INFINITY,
                f64::INFINITY,
                f64::NEG_INFINITY,
            ),
            |(min_x, max_x, min_y, max_y), &(x, y)| {
                (min_x.min(x), max_x.max(x), min_y.min(y), max_y.max(y))
            },
        );
        debug!(
            "[Stroke Check] user raw range: x=[{:.1}, {:.1}], y=[{:.1}, {:.1}]",
            min_x, max_x, min_y, max_y
        );
    }

    if !stroke_samples.is_empty() {
        let (min_x, max_x, min_y, max_y) = stroke_samples.iter().fold(
            (
                f64::INFINITY,
                f64::NEG_INFINITY,
                f64::INFINITY,
                f64::NEG_INFINITY,
            ),
            |(min_x, max_x, min_y, max_y), &(x, y)| {
                (min_x.min(x), max_x.max(x), min_y.min(y), max_y.max(y))
            },
        );
        debug!(
            "[Stroke Check] stroke raw range: x=[{:.1}, {:.1}], y=[{:.1}, {:.1}]",
            min_x, max_x, min_y, max_y
        );
    }

    // Resample reference stroke to fixed number of points
    let stroke_resampled = resample_points(&stroke_samples, SAMPLE_COUNT);

    // Count how many reference points have a nearby user point
    // Use tolerance in canvas pixels (e.g., 25 pixels = ~8% of 320px canvas)
    let mut covered_count = 0;

    for stroke_point in &stroke_resampled {
        let is_covered = user_points.iter().any(|user_point| {
            let dx = user_point.0 - stroke_point.0;
            let dy = user_point.1 - stroke_point.1;
            let distance = (dx * dx + dy * dy).sqrt();
            distance <= CANVAS_TOLERANCE
        });
        if is_covered {
            covered_count += 1;
        }
    }

    let coverage_ratio = covered_count as f64 / stroke_resampled.len() as f64;
    debug!(
        "[Stroke Check] coverage: {}/{} = {:.2}, threshold: {:.2}",
        covered_count,
        stroke_resampled.len(),
        coverage_ratio,
        SUCCESS_THRESHOLD
    );

    let result = coverage_ratio >= SUCCESS_THRESHOLD;
    debug!("[Stroke Check] RESULT: {}", result);

    result
}

fn sample_stroke_path(d: &str) -> Vec<(f64, f64)> {
    let mut points = Vec::new();
    let mut current_pos = (0.0, 0.0); // Keep in SVG coords
    let mut start_pos = (0.0, 0.0); // Keep in SVG coords
    let chars: Vec<char> = d.chars().collect();
    let mut pos = 0;
    let mut current_cmd = 'M';

    while pos < chars.len() {
        let c = chars[pos];
        if c.is_ascii_alphabetic() {
            current_cmd = c;
            pos += 1;
        }
        match current_cmd {
            'M' | 'm' => {
                if let Some((x, y, new_pos)) = parse_coords(&chars, pos) {
                    let (abs_x, abs_y) = if current_cmd == 'm' {
                        (current_pos.0 + x, current_pos.1 + y)
                    } else {
                        (x, y)
                    };
                    current_pos = (abs_x, abs_y); // SVG coords
                    start_pos = current_pos;
                    points.push((abs_x * SVG_SCALE, abs_y * SVG_SCALE)); // Convert to canvas
                    pos = new_pos;
                    current_cmd = if current_cmd == 'm' { 'l' } else { 'L' };
                } else {
                    break;
                }
            }
            'L' | 'l' => {
                if let Some((x, y, new_pos)) = parse_coords(&chars, pos) {
                    let (abs_x, abs_y) = if current_cmd == 'l' {
                        (current_pos.0 + x, current_pos.1 + y) // Both in SVG coords - correct!
                    } else {
                        (x, y)
                    };
                    let start = (current_pos.0 * SVG_SCALE, current_pos.1 * SVG_SCALE);
                    let end = (abs_x * SVG_SCALE, abs_y * SVG_SCALE);
                    interpolate_line(&mut points, start, end);
                    current_pos = (abs_x, abs_y); // SVG coords
                    pos = new_pos;
                } else {
                    break;
                }
            }
            'C' | 'c' => {
                if let Some((x1, y1, x2, y2, x, y, new_pos)) = parse_curve_coords(&chars, pos) {
                    let (abs_x1, abs_y1, abs_x2, abs_y2, abs_x, abs_y) = if current_cmd == 'c' {
                        (
                            current_pos.0 + x1, // All in SVG coords - correct!
                            current_pos.1 + y1,
                            current_pos.0 + x2,
                            current_pos.1 + y2,
                            current_pos.0 + x,
                            current_pos.1 + y,
                        )
                    } else {
                        (x1, y1, x2, y2, x, y)
                    };
                    let start = (current_pos.0 * SVG_SCALE, current_pos.1 * SVG_SCALE);
                    let ctrl1 = (abs_x1 * SVG_SCALE, abs_y1 * SVG_SCALE);
                    let ctrl2 = (abs_x2 * SVG_SCALE, abs_y2 * SVG_SCALE);
                    let end = (abs_x * SVG_SCALE, abs_y * SVG_SCALE);
                    interpolate_bezier(&mut points, start, ctrl1, ctrl2, end);
                    current_pos = (abs_x, abs_y); // SVG coords
                    pos = new_pos;
                } else {
                    break;
                }
            }
            'Z' | 'z' => {
                let start = (current_pos.0 * SVG_SCALE, current_pos.1 * SVG_SCALE);
                let end = (start_pos.0 * SVG_SCALE, start_pos.1 * SVG_SCALE);
                interpolate_line(&mut points, start, end);
                current_pos = start_pos;
                pos += 1;
            }
            _ => {
                pos += 1;
            }
        }
    }
    resample_points(&points, SAMPLE_COUNT)
}
fn interpolate_line(points: &mut Vec<(f64, f64)>, start: (f64, f64), end: (f64, f64)) {
    let steps = 10;
    for i in 1..=steps {
        let t = i as f64 / steps as f64;
        let x = start.0 + (end.0 - start.0) * t;
        let y = start.1 + (end.1 - start.1) * t;
        points.push((x, y));
    }
}
fn interpolate_bezier(
    points: &mut Vec<(f64, f64)>,
    p0: (f64, f64),
    p1: (f64, f64),
    p2: (f64, f64),
    p3: (f64, f64),
) {
    let steps = 20;
    for i in 1..=steps {
        let t = i as f64 / steps as f64;
        let t2 = t * t;
        let t3 = t2 * t;
        let mt = 1.0 - t;
        let mt2 = mt * mt;
        let mt3 = mt2 * mt;
        let x = mt3 * p0.0 + 3.0 * mt2 * t * p1.0 + 3.0 * mt * t2 * p2.0 + t3 * p3.0;
        let y = mt3 * p0.1 + 3.0 * mt2 * t * p1.1 + 3.0 * mt * t2 * p2.1 + t3 * p3.1;
        points.push((x, y));
    }
}
fn resample_points(points: &[(f64, f64)], count: usize) -> Vec<(f64, f64)> {
    if points.is_empty() {
        return Vec::new();
    }
    let mut result = Vec::with_capacity(count);
    let total_points = points.len();
    for i in 0..count {
        let idx = (i * (total_points - 1)) / (count - 1).max(1);
        result.push(points[idx.min(total_points - 1)]);
    }
    result
}
