use super::ocr_processing::{OcrState, ProcessContext, process_file};
use crate::ui_components::{
    Alert, AlertType, Button, ButtonVariant, LoadingStageItem, OcrLoadingStage, OcrLoadingState,
    StageType, Text, TextSize, TypographyVariant, get_stage_info,
};
use leptos::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::js_sys::Function;
use web_sys::{ClipboardEvent, HtmlInputElement};

fn stage_item_view(
    stage: RwSignal<OcrLoadingStage>,
    stage_type: StageType,
    title: &'static str,
) -> impl IntoView {
    move || {
        let info = get_stage_info(&stage.get(), stage_type);
        view! {
            <LoadingStageItem
                status=info.status
                title=title.to_string()
                description=info.description
                progress=info.progress
                error_message=info.error_message
            />
        }
    }
}

#[component]
pub fn ImageInputStage(
    #[prop(optional, into)] class: Signal<String>,
    is_open: RwSignal<bool>,
    on_text_extracted: Callback<String>,
    on_error: Callback<String>,
    on_switch_to_text: Callback<()>,
) -> impl IntoView {
    let ocr_state = RwSignal::new(OcrState::Idle);
    let image_preview = RwSignal::new(None::<String>);
    let error_message = RwSignal::new(None::<String>);
    let is_drag_over = RwSignal::new(false);
    let ocr_loading_state = OcrLoadingState::new();
    let disposed = StoredValue::new(());

    Effect::new(move |_| {
        if !is_open.get() {
            ocr_loading_state.cancel_requested.set(true);
            ocr_loading_state.reset();
            ocr_state.set(OcrState::Idle);
            image_preview.set(None);
            error_message.set(None);
        }
    });

    let ocr_loading_state_for_file = ocr_loading_state;
    let disposed_for_file = disposed;
    let on_file_change = move |ev: leptos::ev::Event| {
        let target = match ev.target() {
            Some(t) => t,
            None => return,
        };
        let input: HtmlInputElement = match target.dyn_into() {
            Ok(i) => i,
            Err(_) => return,
        };
        let files = match input.files() {
            Some(f) => f,
            None => return,
        };

        if let Some(file) = files.get(0) {
            process_file(
                file,
                ProcessContext {
                    image_preview,
                    ocr_state,
                    ocr_loading_state: ocr_loading_state_for_file,
                    error_message,
                    disposed: disposed_for_file,
                },
                on_text_extracted,
                on_error,
            );
        }
    };

    let ocr_loading_state_for_drag = ocr_loading_state;
    let disposed_for_drag = disposed;
    let on_drag_over = move |ev: leptos::ev::DragEvent| {
        ev.prevent_default();
        is_drag_over.set(true);
    };

    let on_drag_leave = move |ev: leptos::ev::DragEvent| {
        ev.prevent_default();
        is_drag_over.set(false);
    };

    let on_drop = move |ev: leptos::ev::DragEvent| {
        ev.prevent_default();
        is_drag_over.set(false);

        if let Some(data_transfer) = ev.data_transfer()
            && let Some(files) = data_transfer.files()
            && let Some(file) = files.get(0)
        {
            process_file(
                file,
                ProcessContext {
                    image_preview,
                    ocr_state,
                    ocr_loading_state: ocr_loading_state_for_drag,
                    error_message,
                    disposed: disposed_for_drag,
                },
                on_text_extracted,
                on_error,
            );
        }
    };

    let ocr_loading_state_for_paste = ocr_loading_state;
    let disposed_for_paste = disposed;

    let stored_closure = StoredValue::new_local(None::<StoredClosure>);

    Effect::new(move |_| {
        let window = match web_sys::window() {
            Some(w) => w,
            None => return,
        };

        let ctx = ProcessContext {
            image_preview,
            ocr_state,
            ocr_loading_state: ocr_loading_state_for_paste,
            error_message,
            disposed: disposed_for_paste,
        };

        let closure = wasm_bindgen::closure::Closure::<dyn FnMut(ClipboardEvent)>::new(
            move |event: ClipboardEvent| {
                if let Some(clipboard_data) = event.clipboard_data()
                    && let Some(files) = clipboard_data.files()
                    && let Some(file) = files.get(0)
                {
                    process_file(file, ctx.clone(), on_text_extracted, on_error);
                }
            },
        );

        let closure_ptr: Function = closure
            .as_ref()
            .dyn_ref::<Function>()
            .cloned()
            .expect("Closure must be convertible to Function");
        if window
            .add_event_listener_with_callback("paste", &closure_ptr)
            .is_ok()
        {
            stored_closure.set_value(Some(StoredClosure {
                window,
                closure_ptr,
                _closure: closure,
            }));
        }
    });

    let ocr_loading_state_for_cancel = ocr_loading_state;
    let ocr_state_for_cancel = ocr_state;
    let on_cancel = move |_| {
        ocr_loading_state_for_cancel.cancel_requested.set(true);
        ocr_state_for_cancel.set(OcrState::Idle);
    };

    let stage = ocr_loading_state.stage;

    view! {
        <div class=move || format!("{} space-y-4", class.get())>
            {move || {
                if matches!(ocr_state.get(), OcrState::Processing) {
                    view! {
                        <div class="space-y-4">
                            <h2 class="text-lg font-semibold text-[var(--fg-black)] flex items-center gap-2">
                                <span class="spinner spinner-sm"></span>
                                "Подготовка к распознаванию"
                            </h2>

                            <div class="space-y-3" role="list">
                                {stage_item_view(stage, StageType::Deim, "Сегментация текста")}
                                {stage_item_view(stage, StageType::Parseq, "Распознавание символов")}
                                {stage_item_view(stage, StageType::Init, "Инициализация моделей")}
                                {stage_item_view(stage, StageType::Recognize, "Распознавание текста")}
                            </div>

                            <div class="flex justify-end pt-2">
                                <Button
                                    variant=Signal::derive(|| ButtonVariant::Ghost)
                                    disabled=Signal::derive(move || ocr_loading_state.cancel_requested.get())
                                    on_click=Callback::new(on_cancel)
                                >
                                    {move || if ocr_loading_state.cancel_requested.get() { "Отмена..." } else { "Отменить" }}
                                </Button>
                            </div>
                        </div>
                    }.into_any()
                } else {
                    view! {
                        <>
                            <div
                                class=move || {
                                    let base = "border-2 border-dashed rounded-lg p-8 text-center transition-colors cursor-pointer";
                                    if is_drag_over.get() {
                                        format!("{} border-[var(--accent-olive)] bg-[var(--accent-olive)]/10", base)
                                    } else {
                                        format!("{} border-[var(--border-light)] hover:border-[var(--accent-olive)]/50", base)
                                    }
                                }
                                on:dragover=on_drag_over
                                on:dragleave=on_drag_leave
                                on:drop=on_drop
                            >
                                <label class="cursor-pointer">
                                    <input
                                        type="file"
                                        accept="image/png,image/jpeg,image/webp"
                                        class="hidden"
                                        on:change=on_file_change
                                    />
                                    <div class="space-y-2">
                                        <svg class="mx-auto h-12 w-12 text-[var(--fg-muted)]" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 16l4.586-4.586a2 2 0 012.828 0L16 16m-2-2l1.586-1.586a2 2 0 012.828 0L20 14m-6-6h.01M6 20h12a2 2 0 002-2V6a2 2 0 00-2-2H6a2 2 0 00-2 2v12a2 2 0 002 2z" />
                                        </svg>
                                        <Text size=TextSize::Default variant=TypographyVariant::Muted>
                                            "Перетащите изображение, вставьте из буфера обмена или нажмите для выбора"
                                        </Text>
                                        <Text size=TextSize::Small variant=TypographyVariant::Muted>
                                            "PNG, JPEG, WebP (макс. 10 MB)"
                                        </Text>
                                    </div>
                                </label>
                            </div>

                            {move || {
                                image_preview.get().map(|src| view! {
                                    <div class="relative">
                                        <img src=src class="max-h-64 mx-auto rounded-lg shadow-md" alt="Preview" />
                                    </div>
                                })
                            }}

                            {move || {
                                error_message.get().map(|msg| view! {
                                    <Alert
                                        alert_type=Signal::derive(|| AlertType::Warning)
                                        title=Signal::derive(|| "Не удалось распознать".to_string())
                                        message=Signal::derive(move || msg.clone())
                                    />
                                    <Button
                                        variant=ButtonVariant::Ghost
                                        on_click=Callback::new(move |_| on_switch_to_text.run(()))
                                    >
                                        "Ввести текст вручную"
                                    </Button>
                                })
                            }}
                        </>
                    }.into_any()
                }
            }}
        </div>
    }
}

struct StoredClosure {
    window: web_sys::Window,
    closure_ptr: Function,
    _closure: wasm_bindgen::closure::Closure<dyn FnMut(ClipboardEvent)>,
}

impl Drop for StoredClosure {
    fn drop(&mut self) {
        let _ = self
            .window
            .remove_event_listener_with_callback("paste", &self.closure_ptr);
    }
}
