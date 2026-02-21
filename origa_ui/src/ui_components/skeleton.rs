use leptos::prelude::*;

#[component]
pub fn Skeleton(
    #[prop(optional, into)] class: Signal<String>,
    #[prop(optional, into)] width: Signal<Option<String>>,
    #[prop(optional, into)] height: Signal<Option<String>>,
) -> impl IntoView {
    let style = move || {
        let mut styles = vec![];
        if let Some(w) = width.get() {
            styles.push(format!("width: {}", w));
        }
        if let Some(h) = height.get() {
            styles.push(format!("height: {}", h));
        }
        styles.join("; ")
    };

    view! {
        <div class=move || format!("skeleton {}", class.get()) style=style></div>
    }
}
