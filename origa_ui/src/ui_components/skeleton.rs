use leptos::prelude::*;

#[component]
pub fn Skeleton(
    #[prop(optional, into)] class: String,
    #[prop(optional, into)] width: Option<String>,
    #[prop(optional, into)] height: Option<String>,
) -> impl IntoView {
    let base_class = "skeleton";
    let full_class = format!("{} {}", base_class, class);

    let style = move || {
        let mut styles = vec![];
        if let Some(w) = width.as_ref() {
            styles.push(format!("width: {}", w));
        }
        if let Some(h) = height.as_ref() {
            styles.push(format!("height: {}", h));
        }
        styles.join("; ")
    };

    view! {
        <div class=full_class style=style></div>
    }
}
