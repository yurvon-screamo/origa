use leptos::prelude::*;

/// A real app screenshot with the double-sheet frame: quiet and straight.
/// No window chrome, no rotation — the capture speaks for itself.
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
            <img src=src class="shot__img" alt=img_alt/>
            {(!label.is_empty()).then(|| {
                view! { <figcaption class="shot__caption">{label}</figcaption> }
            })}
        </figure>
    }
}
