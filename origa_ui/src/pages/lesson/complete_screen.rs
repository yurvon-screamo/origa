use crate::ui_components::{
    Button, ButtonVariant, Card, DisplayText, Text, TextSize, TypographyVariant,
};
use leptos::prelude::*;
use leptos_router::hooks::use_navigate;
use leptos_use::use_event_listener;

#[component]
pub fn LessonCompleteScreen(is_completed: RwSignal<bool>, review_count: usize) -> impl IntoView {
    let navigate = use_navigate();

    let navigate_next_lesson = {
        let navigate = navigate.clone();
        Callback::new(move |_: ()| {
            navigate("/lesson", Default::default());
        })
    };

    let navigate_home = {
        let navigate = navigate.clone();
        Callback::new(move |_: ()| {
            navigate("/home", Default::default());
        })
    };

    let navigate_kb_next = navigate.clone();
    let navigate_kb_home = navigate;
    let _ = use_event_listener(document(), leptos::ev::keydown, move |ev| {
        if !is_completed.get() {
            return;
        }

        match ev.key().as_str() {
            "Enter" | " " => {
                if ev.key() == " " {
                    ev.prevent_default();
                }
                navigate_kb_next("/lesson", Default::default());
            }
            "Escape" => {
                navigate_kb_home("/home", Default::default());
            }
            _ => {}
        }
    });

    view! {
        <div class="text-center py-8">
            <Card class=Signal::derive(|| "p-6 mb-6".to_string())>
                <div class="grid grid-cols-1 gap-4">
                    <div>
                        <Text size=TextSize::Small variant=TypographyVariant::Muted uppercase=true>
                            "Пройдено"
                        </Text>
                        <DisplayText>
                            {review_count}
                        </DisplayText>
                    </div>
                </div>
            </Card>

            <div class="flex gap-3 justify-center">
                <Button
                    variant=Signal::derive(|| ButtonVariant::Filled)
                    on_click=Callback::new(move |_: leptos::ev::MouseEvent| {
                        navigate_next_lesson.run(());
                    })
                >
                    "Следующий урок"
                </Button>

                <Button
                    variant=Signal::derive(|| ButtonVariant::Ghost)
                    on_click=Callback::new(move |_: leptos::ev::MouseEvent| {
                        navigate_home.run(());
                    })
                >
                    "На главную"
                </Button>
            </div>
        </div>
    }
}
