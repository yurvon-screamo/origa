use crate::ui_components::{
    Button, ButtonVariant, Card, DisplayText, Text, TextSize, TypographyVariant,
};
use leptos::prelude::*;
use leptos_router::components::A;

#[component]
pub fn LessonCompleteScreen(new_count: usize, review_count: usize) -> impl IntoView {
    view! {
        <div class="text-center py-8">
            <DisplayText class="mb-4">
                "Урок завершён!"
            </DisplayText>

            <Card class=Signal::derive(|| "p-6 mb-6".to_string())>
                <div class="grid grid-cols-2 gap-4">
                    <div>
                        <Text size=TextSize::Small variant=TypographyVariant::Muted uppercase=true>
                            "Новых"
                        </Text>
                        <DisplayText>
                            {new_count}
                        </DisplayText>
                    </div>
                    <div>
                        <Text size=TextSize::Small variant=TypographyVariant::Muted uppercase=true>
                            "Повторено"
                        </Text>
                        <DisplayText>
                            {review_count}
                        </DisplayText>
                    </div>
                </div>
            </Card>

            <A href="/home">
                <Button variant=Signal::derive(|| ButtonVariant::Filled)>
                    "На главную"
                </Button>
            </A>
        </div>
    }
}
