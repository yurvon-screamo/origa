use crate::core::config::public_url;
use crate::i18n::{t, use_i18n};
use futures::future::{AbortHandle, abortable};
use leptos::prelude::*;
use leptos::task::spawn_local;
use std::sync::{Arc, Mutex};

#[derive(Clone, Copy, PartialEq, Default)]
pub enum KanjiViewMode {
    #[default]
    Animation,
    Frames,
}

fn add_animation_delays(svg_html: &str, stroke_time: f32) -> (String, usize) {
    let mut result = String::with_capacity(svg_html.len() + 1000);
    let mut stroke_index = 0usize;
    let mut pos = 0;

    while let Some(rel_start) = svg_html[pos..].find("<path") {
        let abs_start = pos + rel_start;
        result.push_str(&svg_html[pos..abs_start]);

        let rest = &svg_html[abs_start..];
        let tag_end = rest.find('>').unwrap();
        let path_tag = &rest[..=tag_end];

        if path_tag.contains("class=\"bg\"") || path_tag.contains("class='bg'") {
            result.push_str(path_tag);
        } else {
            let delay = stroke_index as f32 * stroke_time;
            let style_attr = format!(" style=\"animation-delay:{:.3}s\"", delay);
            let insert_pos = if path_tag.ends_with("/>") {
                path_tag.len() - 2
            } else {
                path_tag.len() - 1
            };
            result.push_str(&path_tag[..insert_pos]);
            result.push_str(&style_attr);
            result.push_str(&path_tag[insert_pos..]);
            stroke_index += 1;
        }

        pos = abs_start + tag_end + 1;
    }
    result.push_str(&svg_html[pos..]);
    (result, stroke_index)
}

#[component]
pub fn KanjiAnimation(
    kanji: String,
    #[prop(optional)] mode: KanjiViewMode,
    #[prop(optional, into)] test_id: Signal<String>,
    #[prop(into)] fallback: Option<String>,
) -> impl IntoView {
    let i18n = use_i18n();
    let (iteration, set_iteration) = signal(0);
    let abort_handle = Arc::new(Mutex::new(None::<AbortHandle>));
    let disposed = StoredValue::new(());

    let encoded = urlencoding::encode(&kanji);
    let svg_path = match mode {
        KanjiViewMode::Animation => {
            public_url(&format!("/public/kanji_animations/{}.svg", encoded))
        },
        KanjiViewMode::Frames => public_url(&format!("/public/kanji_frames/{}.svg", encoded)),
    };

    let container_class = match mode {
        KanjiViewMode::Animation => "kanji-animation-container kanji-animation-svg",
        KanjiViewMode::Frames => "kanji-animation-container kanji-frames-svg",
    };

    let svg_content = LocalResource::new(move || {
        let path = svg_path.clone();

        async move {
            use leptos::wasm_bindgen::JsCast;
            use wasm_bindgen_futures::JsFuture;

            let window = web_sys::window()?;
            let resp = JsFuture::from(window.fetch_with_str(&path)).await.ok()?;
            let response = resp.dyn_into::<web_sys::Response>().ok()?;
            if !response.ok() {
                return None;
            }
            let text = JsFuture::from(response.text().ok()?).await.ok()?;
            text.as_string()
        }
    });

    let stroke_time = 0.4f32;
    let abort_handle_clone = abort_handle.clone();

    Effect::new(move |_| {
        // Cancel previous timer if exists
        if let Some(handle) = abort_handle_clone.lock().unwrap().take() {
            handle.abort();
        }

        let iter = iteration.get();
        let abort_handle_clone2 = abort_handle_clone.clone();

        if iter % 2 != 0 {
            spawn_local(async move {
                let future = async {
                    gloo_timers::future::TimeoutFuture::new(1500).await;
                    if disposed.is_disposed() {
                        return;
                    }
                    set_iteration.try_update(|n| *n += 1);
                };
                let (abortable, handle) = abortable(future);
                *abort_handle_clone2.lock().unwrap() = Some(handle);
                let _ = abortable.await;
            });
        } else if let Some(Some(svg_html)) = svg_content.get()
            && matches!(mode, KanjiViewMode::Animation)
        {
            let bg_count = svg_html.matches("class=\"bg\"").count();
            let path_count = svg_html.matches("<path").count();
            let strokes = path_count.saturating_sub(bg_count).max(1);
            let total_duration_ms = ((strokes as f32 * stroke_time + 0.5) * 1000.0) as u32;

            spawn_local(async move {
                let future = async {
                    gloo_timers::future::TimeoutFuture::new(total_duration_ms).await;
                    if disposed.is_disposed() {
                        return;
                    }
                    set_iteration.try_update(|n| *n += 1);
                };
                let (abortable, handle) = abortable(future);
                *abort_handle_clone2.lock().unwrap() = Some(handle);
                let _ = abortable.await;
            });
        }
    });

    let abort_handle_cleanup = abort_handle.clone();
    on_cleanup(move || {
        if let Some(handle) = abort_handle_cleanup.lock().unwrap().take() {
            handle.abort();
        }
    });

    let test_id_val = move || {
        let val = test_id.get();
        if val.is_empty() { None } else { Some(val) }
    };

    view! {
        <div data-testid=test_id_val class={container_class}>
            <Suspense fallback=move || view! { <div class="kanji-loading">{t!(i18n, ui.loading_animation)}</div> }>
                {move || {
                    if iteration.get() % 2 != 0 {
                        return None;
                    }

                    let svg_content_state = svg_content.get();
                    let is_error = svg_content_state == Some(None);
                    let svg_result = svg_content_state.flatten();

                    if is_error {
                        fallback.as_ref().map(|text| {
                            view! {
                                <div class="kanji-fallback">{text.to_string()}</div>
                            }
                            .into_any()
                        })
                    } else {
                        svg_result.map(move |svg_html: String| {
                            let (modified_svg, _strokes) = add_animation_delays(&svg_html, stroke_time);
                            view! {
                                <div inner_html={modified_svg} />
                            }
                            .into_any()
                        })
                    }
                }}
            </Suspense>
        </div>
    }
}

#[component]
pub fn KanjiWritingSection(
    kanji: String,
    #[prop(optional)] mode: KanjiViewMode,
    #[prop(optional, into)] test_id: Signal<String>,
    #[prop(optional)] fallback: Option<String>,
) -> impl IntoView {
    let test_id_val = move || {
        let val = test_id.get();
        if val.is_empty() { None } else { Some(val) }
    };

    view! {
        <div data-testid=test_id_val class="kanji-writing-section">
            <div class="kanji-writing-grid">
                <KanjiAnimation
                    kanji={kanji.clone()}
                    mode={mode}
                    test_id="kanji-animation"
                    fallback={fallback.clone()}
                />
            </div>
        </div>
    }
}
