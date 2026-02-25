use leptos::prelude::*;

#[derive(Clone, Copy, PartialEq, Default)]
pub enum KanjiViewMode {
    #[default]
    Animation,
    Frames,
}

#[component]
pub fn KanjiAnimation(kanji: String, #[prop(optional)] mode: KanjiViewMode) -> impl IntoView {
    let encoded = urlencoding::encode(&kanji);
    let svg_path = match mode {
        KanjiViewMode::Animation => format!("/kanji_animations/{}.svg", encoded),
        KanjiViewMode::Frames => format!("/kanji_frames/{}.svg", encoded),
    };

    let container_class = match mode {
        KanjiViewMode::Animation => "kanji-animation-container kanji-animation-svg",
        KanjiViewMode::Frames => "kanji-animation-container kanji-frames-svg",
    };

    let svg_content = Resource::new(
        move || svg_path.clone(),
        |path| async move {
            #[cfg(target_arch = "wasm32")]
            {
                use wasm_bindgen_futures::JsFuture;
                use web_sys::Request;

                let window = web_sys::window()?;
                let resp = JsFuture::from(window.fetch_with_str(&path)).await.ok()?;
                let text = JsFuture::from(resp.dyn_into::<web_sys::Response>().ok()?.text()?)
                    .await
                    .ok()?;
                text.as_string()
            }
            #[cfg(not(target_arch = "wasm32"))]
            {
                let _ = path;
                None
            }
        },
    );

    view! {
        <div class={container_class}>
            <Suspense fallback=|| view! { <div class="kanji-loading">"Загрузка..."</div> }>
                {move || {
                    svg_content.get().flatten().map(|svg_html: String| {
                        view! { <div inner_html={svg_html} /> }
                    })
                }}
            </Suspense>
        </div>
    }
}

#[component]
pub fn KanjiWritingSection(kanji: String, #[prop(optional)] show_frames: bool) -> impl IntoView {
    view! {
        <div class="kanji-writing-section">
            <div class="kanji-writing-title">
                "Написание"
            </div>
            <div class="kanji-writing-grid">
                <KanjiAnimation
                    kanji={kanji.clone()}
                    mode={KanjiViewMode::Animation}
                />
                {move || if show_frames {
                    Some(view! {
                        <KanjiAnimation
                            kanji={kanji.clone()}
                            mode={KanjiViewMode::Frames}
                        />
                    })
                } else {
                    None
                }}
            </div>
        </div>
    }
}
