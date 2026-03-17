use super::lesson_state::LessonContext;
use crate::repository::set_last_sync_time;
use crate::ui_components::{
    Button, ButtonVariant, Card, DisplayText, Text, TextSize, ToastContainer, ToastData, ToastType,
    TypographyVariant,
};
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_router::hooks::use_navigate;
use leptos_use::use_event_listener;

const SYNC_TOAST_ID: usize = usize::MAX;

#[component]
pub fn LessonCompleteScreen(is_completed: RwSignal<bool>, review_count: usize) -> impl IntoView {
    let navigate = use_navigate();
    let lesson_ctx = use_context::<LessonContext>().expect("lesson context");
    let is_disposed = use_context::<StoredValue<bool>>().expect("is_disposed must be provided");
    let is_syncing = RwSignal::new(false);
    let toasts: RwSignal<Vec<ToastData>> = RwSignal::new(Vec::new());

    let sync_with_server = {
        let lesson_ctx = lesson_ctx.clone();
        move || {
            if is_syncing.get_untracked() {
                return;
            }
            is_syncing.set(true);

            toasts.update(|t| {
                t.push(ToastData {
                    id: SYNC_TOAST_ID,
                    toast_type: ToastType::Info,
                    title: "Синхронизация".to_string(),
                    message: "Сохранение прогресса...".to_string(),
                    duration_ms: None,
                });
            });

            let repo = lesson_ctx.repository.clone();
            let toasts = toasts;
            spawn_local(async move {
                match repo.merge_current_user().await {
                    Ok(()) => {
                        if is_disposed.get_value() {
                            return;
                        }
                        set_last_sync_time(js_sys::Date::now() as u64 / 1000);
                        toasts.update(|t| t.retain(|toast| toast.id != SYNC_TOAST_ID));
                        toasts.update(|t| {
                            t.push(ToastData {
                                id: t.len(),
                                toast_type: ToastType::Success,
                                title: "Сохранено".to_string(),
                                message: "Прогресс успешно сохранён".to_string(),
                                duration_ms: Some(3000),
                            });
                        });
                        tracing::info!("Lesson complete: sync successful");
                    }
                    Err(e) => {
                        if is_disposed.get_value() {
                            return;
                        }
                        toasts.update(|t| t.retain(|toast| toast.id != SYNC_TOAST_ID));
                        toasts.update(|t| {
                            t.push(ToastData {
                                id: t.len(),
                                toast_type: ToastType::Error,
                                title: "Ошибка синхронизации".to_string(),
                                message: e.to_string(),
                                duration_ms: Some(5000),
                            });
                        });
                        tracing::error!("Lesson complete: sync error: {:?}", e);
                    }
                }
                if is_disposed.get_value() {
                    return;
                }
                is_syncing.set(false);
            });
        }
    };

    let go_next_lesson = {
        let lesson_ctx = lesson_ctx.clone();
        let sync_with_server = sync_with_server.clone();
        Callback::new(move |_: ()| {
            sync_with_server();
            lesson_ctx.is_completed.set(false);
            lesson_ctx.reload_trigger.update(|t| *t += 1);
        })
    };

    let go_home = {
        let navigate = navigate.clone();
        let sync_with_server = sync_with_server.clone();
        Callback::new(move |_: ()| {
            sync_with_server();
            navigate("/home", Default::default());
        })
    };

    let kb_lesson_ctx = lesson_ctx.clone();
    let kb_navigate = navigate;
    let kb_sync = sync_with_server.clone();
    let _ = use_event_listener(document(), leptos::ev::keydown, move |ev| {
        if !is_completed.get() {
            return;
        }

        match ev.key().as_str() {
            "Enter" | " " => {
                if ev.key() == " " {
                    ev.prevent_default();
                }
                kb_sync();
                kb_lesson_ctx.is_completed.set(false);
                kb_lesson_ctx.reload_trigger.update(|t| *t += 1);
            }
            "Escape" => {
                kb_sync();
                kb_navigate("/home", Default::default());
            }
            _ => {}
        }
    });

    view! {
        <ToastContainer toasts=toasts duration_ms=5000 />

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
                    "Следующий урок" <span class="hidden sm:inline">"[Пробел]"</span>
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
