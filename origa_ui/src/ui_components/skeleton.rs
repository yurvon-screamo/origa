use leptos::prelude::*;

#[component]
pub fn Skeleton(
    #[prop(optional)] class: Option<String>,
    #[prop(optional)] width: Option<String>,
    #[prop(optional)] height: Option<String>,
) -> impl IntoView {
    view! {
        <div
            class={"anima-skeleton-paper ".to_string() + &class.unwrap_or_default()}
            style=move || {
                let mut styles = vec![];
                if let Some(ref w) = width {
                    styles.push(format!("width: {}", w));
                }
                if let Some(ref h) = height {
                    styles.push(format!("height: {}", h));
                }
                styles.join("; ")
            }
        ></div>
    }
}
