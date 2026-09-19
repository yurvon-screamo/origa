use leptos::prelude::*;

/// A real app screenshot presented as a minimal window: a thin title bar
/// with three square dots and a mono label, then the capture itself.
///
/// The composition layer (chrome, frame, offset-shadow backing sheet) is
/// HTML/CSS so it stays crisp at any DPI and every string in it is
/// localizable — the bitmap inside is the only non-translatable part.
/// Decorative by default (aria-hidden): the surrounding copy carries the
/// meaning, the shot is evidence.
#[component]
pub fn Shot(
    src: String,
    #[prop(default = String::new())] label: String,
    #[prop(default = "shot--desktop".to_string())] variant: String,
    #[prop(optional)] img_alt: Option<String>,
) -> impl IntoView {
    view! {
        <figure class=format!("shot {variant}") aria-hidden="true">
            <div class="shot__bar">
                <span class="shot__dot shot__dot--a"></span>
                <span class="shot__dot shot__dot--b"></span>
                <span class="shot__dot shot__dot--c"></span>
                {(!label.is_empty()).then(|| {
                    view! { <span class="shot__bar-label">{label}</span> }
                })}
            </div>
            <img src=src class="shot__img" alt=img_alt/>
        </figure>
    }
}
