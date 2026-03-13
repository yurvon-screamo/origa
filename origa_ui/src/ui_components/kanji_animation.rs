use leptos::prelude::*;

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
pub fn KanjiAnimation(kanji: String, #[prop(optional)] mode: KanjiViewMode) -> impl IntoView {
    let (iteration, set_iteration) = signal(0);

    let encoded = urlencoding::encode(&kanji);
    let svg_path = match mode {
        KanjiViewMode::Animation => crate::config::public_url(&format!("/public/kanji_animations/{}.svg", encoded)),
        KanjiViewMode::Frames => crate::config::public_url(&format!("/public/kanji_frames/{}.svg", encoded)),
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
            let text = JsFuture::from(resp.dyn_into::<web_sys::Response>().ok()?.text().ok()?)
                .await
                .ok()?;
            text.as_string()
        }
    });

    let stroke_time = 0.4f32;

    Effect::new(move |_| {
        use std::time::Duration;

        let iter = iteration.get();
        if iter % 2 != 0 {
            set_timeout(
                move || set_iteration.update(|n| *n += 1),
                Duration::from_millis(1500),
            );
        } else if let Some(Some(svg_html)) = svg_content.get()
            && matches!(mode, KanjiViewMode::Animation)
        {
            let bg_count = svg_html.matches("class=\"bg\"").count();
            let path_count = svg_html.matches("<path").count();
            let strokes = path_count.saturating_sub(bg_count).max(1);
            let total_duration = strokes as f32 * stroke_time + 0.5;

            set_timeout(
                move || set_iteration.update(|n| *n += 1),
                Duration::from_secs_f32(total_duration),
            );
        }
    });

    view! {
        <div class={container_class}>
            <Suspense fallback=|| view! { <div class="kanji-loading">"Загрузка..."</div> }>
                {move || {
                    if iteration.get() % 2 != 0 {
                        return None;
                    }

                    svg_content.get().flatten().map(move |svg_html: String| {
                        let (modified_svg, _strokes) = add_animation_delays(&svg_html, stroke_time);

                        view! {
                            <div inner_html={modified_svg} />
                        }
                    })
                }}
            </Suspense>
        </div>
    }
}

#[component]
pub fn KanjiWritingSection(kanji: String, #[prop(optional)] mode: KanjiViewMode) -> impl IntoView {
    view! {
        <div class="kanji-writing-section">
            <div class="kanji-writing-grid">
                <KanjiAnimation
                    kanji={kanji.clone()}
                    mode={mode}
                />
            </div>
        </div>
    }
}
