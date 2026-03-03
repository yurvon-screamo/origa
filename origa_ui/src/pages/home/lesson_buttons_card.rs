use crate::ui_components::{Button, ButtonVariant, Card, Text, TextSize, TypographyVariant};
use leptos::prelude::*;
use leptos_router::components::A;

#[component]
pub fn LessonButtonsCard() -> impl IntoView {
    view! {
        <Card class=Signal::derive(|| "p-6 flex flex-col justify-between".to_string())>
            <Text size=TextSize::Small variant=TypographyVariant::Muted class="mb-3">
                "Обучение"
            </Text>
            <div class="flex flex-col gap-2">
                <A href="/lesson">
                    <Button variant=Signal::derive(|| ButtonVariant::Filled) class="w-full">
                        "Урок"
                    </Button>
                </A>
                <A href="/lesson?mode=fixation">
                    <Button variant=Signal::derive(|| ButtonVariant::Olive) class="w-full">
                        "Сложные"
                    </Button>
                </A>
            </div>
        </Card>
    }
}
