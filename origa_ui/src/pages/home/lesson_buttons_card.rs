use crate::ui_components::{Button, ButtonVariant, Card, LabelFrame};
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

    view! {
        <Card shadow=Signal::from(true) class=Signal::derive(|| "flex flex-col justify-center".to_string()) test_id=test_id>
            <LabelFrame test_id=Signal::derive(|| "lesson-frame".to_string())>
                <div class="flex flex-col gap-3">
                    <A href="/lesson">
                        <Button variant=Signal::from(ButtonVariant::Filled) class="w-full py-2" test_id=test_id_lesson>
                            "Урок"
                        </Button>
                    </A>
                    <A href="/lesson?mode=fixation">
                        <Button variant=Signal::from(ButtonVariant::Olive) class="w-full py-2" test_id=test_id_fixation>
                            "Сложные"
                        </Button>
                    </A>
                </div>
            </LabelFrame>
        </Card>
    }
}
