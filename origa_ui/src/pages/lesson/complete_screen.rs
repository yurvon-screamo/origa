use super::lesson_state::LessonContext;
use crate::ui_components::{
    Button, ButtonVariant, Card, DisplayText, Text, TextSize, TypographyVariant,
};
use leptos::prelude::*;
use leptos_router::hooks::use_navigate;
use leptos_use::use_event_listener;

#[component]
pub fn LessonCompleteScreen(is_completed: RwSignal<bool>, review_count: usize) -> impl IntoView {
    let navigate = use_navigate();
    let lesson_ctx = use_context::<LessonContext>().expect("lesson context");

    let go_next_lesson = {
        let lesson_ctx = lesson_ctx.clone();
        Callback::new(move |_: ()| {
            lesson_ctx.is_completed.set(false);
            lesson_ctx.reload_trigger.update(|t| *t += 1);
        })
    };

    let go_home = {
        let navigate = navigate.clone();
        Callback::new(move |_: ()| {
            navigate("/home", Default::default());
        })
    };

    let kb_lesson_ctx = lesson_ctx.clone();
    let kb_navigate = navigate;
    let _ = use_event_listener(document(), leptos::ev::keydown, move |ev| {
        if !is_completed.get() {
            return;
        }

        match ev.key().as_str() {
            "Enter" | " " => {
                if ev.key() == " " {
                    ev.prevent_default();
                }
                kb_lesson_ctx.is_completed.set(false);
                kb_lesson_ctx.reload_trigger.update(|t| *t += 1);
            }
            "Escape" => {
                kb_navigate("/home", Default::default());
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
                        go_next_lesson.run(());
                    })
                >
                    "Следующий урок" <span class="hidden sm:inline">"[Space]"</span>
                </Button>

                <Button
                    variant=Signal::derive(|| ButtonVariant::Ghost)
                    on_click=Callback::new(move |_: leptos::ev::MouseEvent| {
                        go_home.run(());
                    })
                >
                    "На главную" <span class="hidden sm:inline">"[Esc]"</span>
                </Button>
            </div>
        </div>
    }
}
