use crate::ui_components::{Heading, HeadingLevel, Text, TextSize, TypographyVariant};
use leptos::prelude::*;

#[component]
pub fn LoginHeader(#[prop(optional, into)] test_id: Signal<String>) -> impl IntoView {
    let test_id_val = move || {
        let val = test_id.get();
        if val.is_empty() {
            None
        } else {
            Some(val)
        }
    };

    let title_test_id = Signal::derive(move || {
        let val = test_id.get();
        if val.is_empty() {
            "login-title".to_string()
        } else {
            format!("{}-title", val)
        }
    });

    let subtitle_test_id = Signal::derive(move || {
        let val = test_id.get();
        if val.is_empty() {
            "login-subtitle".to_string()
        } else {
            format!("{}-subtitle", val)
        }
    });

    view! {
        <div class="text-center mb-10" data-testid=test_id_val>
            <Heading level=HeadingLevel::H1 variant=TypographyVariant::Primary class="mb-3 whitespace-nowrap" test_id=title_test_id>
                "オリガ"
            </Heading>
            <Text size=TextSize::Small variant=TypographyVariant::Muted uppercase=true tracking_widest=true test_id=subtitle_test_id>
                "Изучение японского языка"
            </Text>
        </div>
    }
}
