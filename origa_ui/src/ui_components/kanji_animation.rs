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
        KanjiViewMode::Animation => format!("/public/kanji_animations/{}.svg", encoded),
        KanjiViewMode::Frames => format!("/public/kanji_frames/{}.svg", encoded),
    };

    let container_class = match mode {
        KanjiViewMode::Animation => "kanji-animation-container",
        KanjiViewMode::Frames => "kanji-animation-container",
    };

    let svg_class = match mode {
        KanjiViewMode::Animation => "kanji-animation-svg",
        KanjiViewMode::Frames => "kanji-frames-svg",
    };

    view! {
        <div class={container_class}>
            <img
                src={svg_path}
                alt={format!("Kanji {} writing", kanji)}
                class={svg_class}
            />
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
