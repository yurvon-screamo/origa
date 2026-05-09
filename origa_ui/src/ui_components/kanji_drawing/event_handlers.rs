use super::{
    CANVAS_SIZE, CanvasContext, DrawingStateRef, MIN_STROKE_POINTS, StrokeData, USER_LINE_WIDTH,
};
use crate::ui_components::kanji_drawing::canvas_renderer::{get_user_color, redraw_canvas};
use crate::ui_components::kanji_drawing::stroke_comparison::is_stroke_similar;
use leptos::ev::PointerEvent;
use leptos::html::Canvas;
use leptos::prelude::*;
use leptos::wasm_bindgen::JsCast;
use web_sys::CanvasRenderingContext2d;

#[derive(Clone)]
pub(super) struct StrokeCompletionState {
    pub strokes: RwSignal<Vec<StrokeData>>,
    pub current_stroke_index: RwSignal<usize>,
    pub is_completed: RwSignal<bool>,
    pub on_complete: Option<Callback<()>>,
}

fn extract_point(ev: &PointerEvent, canvas: &web_sys::HtmlCanvasElement) -> (f64, f64) {
    let rect = canvas.get_bounding_client_rect();
    let x = ev.client_x() as f64 - rect.left();
    let y = ev.client_y() as f64 - rect.top();
    (x, y)
}

fn get_canvas_ctx(
    canvas_ref: &NodeRef<Canvas>,
    ctx_store: &CanvasContext,
) -> Option<(web_sys::HtmlCanvasElement, CanvasRenderingContext2d)> {
    let canvas = canvas_ref.get()?;
    let canvas: web_sys::HtmlCanvasElement = canvas.unchecked_into();
    let ctx_guard = ctx_store.lock().ok()?;
    let ctx = ctx_guard.as_ref()?.clone();
    Some((canvas, ctx))
}

pub(super) fn on_pointer_down(
    ev: PointerEvent,
    canvas_ref: &NodeRef<Canvas>,
    ctx_store: &CanvasContext,
    state: &DrawingStateRef,
) {
    let Some((canvas, ctx)) = get_canvas_ctx(canvas_ref, ctx_store) else {
        return;
    };
    let (x, y) = extract_point(&ev, &canvas);
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

pub(super) fn on_pointer_move(
    ev: PointerEvent,
    canvas_ref: &NodeRef<Canvas>,
    ctx_store: &CanvasContext,
    state: &DrawingStateRef,
) {
    {
        let state_guard = state.lock().ok();
        let Some(ref state_guard) = state_guard else {
            return;
        };
        if !state_guard.is_drawing {
            return;
        }
    }
    let Some((canvas, ctx)) = get_canvas_ctx(canvas_ref, ctx_store) else {
        return;
    };
    let (x, y) = extract_point(&ev, &canvas);
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

pub(super) fn on_pointer_up(
    state: &DrawingStateRef,
    ctx_store: &CanvasContext,
    completion: &StrokeCompletionState,
) {
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
        completion.current_stroke_index.update(|_| {});
        return;
    }
    let stroke_list = completion.strokes.get();
    let current_idx = completion.current_stroke_index.get();
    if current_idx >= stroke_list.len() {
        return;
    }
    let current_stroke = &stroke_list[current_idx];
    if is_stroke_similar(&points, &current_stroke.d) {
        let next_idx = current_idx + 1;
        if next_idx >= stroke_list.len() {
            completion.current_stroke_index.set(next_idx);
            completion.is_completed.set(true);
            if let Some(cb) = &completion.on_complete {
                cb.run(());
            }
        } else {
            completion.current_stroke_index.set(next_idx);
        }
    } else if let Ok(ctx_guard) = ctx_store.lock()
        && let Some(ctx) = ctx_guard.as_ref()
    {
        redraw_canvas(
            ctx,
            &completion.strokes.get(),
            completion.current_stroke_index.get(),
            CANVAS_SIZE,
        );
    }
}
