use crate::i18n::use_i18n;
use crate::ui_components::{Text, TextSize, TypographyVariant};
use leptos::prelude::*;

#[component]
pub fn SelectedCount(
    count: Signal<usize>,
    #[prop(optional, into)] label: Signal<String>,
) -> impl IntoView {
    let i18n = use_i18n();
    move || {
        let c = count.get();
        let label_text = label.get();
        if c > 0 {
            let display = if label_text.is_empty() {
                i18n.get_keys()
                    .ui()
                    .selected_count()
                    .inner()
                    .to_string()
                    .replacen("{}", &c.to_string(), 1)
            } else {
                i18n.get_keys()
                    .ui()
                    .selected_count_with_label()
                    .inner()
                    .to_string()
                    .replacen("{}", &c.to_string(), 1)
                    .replacen("{}", &label_text, 1)
            };
            view! {
                <Text size=TextSize::Small variant=TypographyVariant::Muted>
                    {display}
                </Text>
            }
            .into_any()
        } else {
            ().into_any()
        }
    }
}
