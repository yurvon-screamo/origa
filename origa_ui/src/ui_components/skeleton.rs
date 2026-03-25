use leptos::prelude::*;

#[component]
pub fn Skeleton(
    #[prop(optional)] class: Option<String>,
    #[prop(optional)] width: Option<String>,
    #[prop(optional)] height: Option<String>,
    #[prop(optional, into)] test_id: Signal<String>,
) -> impl IntoView {
    let test_id_val = move || {
        let val = test_id.get();
        if val.is_empty() {
            None
        } else {
            Some(val)
        }
    };

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
            data-testid=test_id_val
        ></div>
    }
}
