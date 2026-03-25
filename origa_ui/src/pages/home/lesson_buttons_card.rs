use crate::ui_components::{Button, ButtonVariant, Card, Text, TextSize, TypographyVariant};
use leptos::prelude::*;
use leptos_router::components::A;

#[component]
pub fn LessonButtonsCard(#[prop(optional, into)] test_id: Signal<String>) -> impl IntoView {
    let test_id_lesson = Signal::derive(move || {
        let val = test_id.get();
        if val.is_empty() {
            "lesson-buttons-lesson".to_string()
        } else {
            format!("{}-lesson", val)
        }
    });

    let test_id_fixation = Signal::derive(move || {
        let val = test_id.get();
        if val.is_empty() {
            "lesson-buttons-fixation".to_string()
        } else {
            format!("{}-fixation", val)
        }
    });

    let test_id_for_card = Signal::derive(move || test_id.get());

    let test_id_for_title = Signal::derive(move || {
        let val = test_id.get();
        if val.is_empty() {
            "lesson-title".to_string()
        } else {
            format!("{}-title", val)
        }
    });

    view! {
        <Card class=Signal::derive(|| "p-6 flex flex-col justify-between".to_string()) test_id=test_id_for_card>
            <Text
                size=TextSize::Small
                variant=TypographyVariant::Muted
                class="mb-3"
                test_id=test_id_for_title
            >
                "Обучение"
            </Text>
            <div class="flex flex-col gap-2">
                <A href="/lesson">
                    <Button variant=Signal::derive(|| ButtonVariant::Filled) class="w-full" test_id=test_id_lesson>
                        "Урок"
                    </Button>
                </A>
                <A href="/lesson?mode=fixation">
                    <Button variant=Signal::derive(|| ButtonVariant::Olive) class="w-full" test_id=test_id_fixation>
                        "Сложные"
                    </Button>
                </A>
            </div>
        </Card>
    }
}
