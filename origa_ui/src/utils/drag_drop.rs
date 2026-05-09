use leptos::prelude::*;
use std::sync::Arc;

#[derive(Copy, Clone)]
pub struct DragDropState {
    is_drag_over: RwSignal<bool>,
    on_drop_file: StoredValue<Arc<dyn Fn(web_sys::File) + Send + Sync>>,
}

impl DragDropState {
    pub fn is_drag_over(self) -> RwSignal<bool> {
        self.is_drag_over
    }

    pub fn on_drag_over(self) -> impl Fn(leptos::ev::DragEvent) + Copy + 'static {
        let is_drag_over = self.is_drag_over;
        move |ev: leptos::ev::DragEvent| {
            ev.prevent_default();
            is_drag_over.set(true);
        }
    }

    pub fn on_drag_leave(self) -> impl Fn(leptos::ev::DragEvent) + Copy + 'static {
        let is_drag_over = self.is_drag_over;
        move |ev: leptos::ev::DragEvent| {
            ev.prevent_default();
            is_drag_over.set(false);
        }
    }

    pub fn on_drop(self) -> impl Fn(leptos::ev::DragEvent) + Copy + 'static {
        let is_drag_over = self.is_drag_over;
        let on_drop_file = self.on_drop_file;
        move |ev: leptos::ev::DragEvent| {
            ev.prevent_default();
            is_drag_over.set(false);
            if let Some(data_transfer) = ev.data_transfer()
                && let Some(files) = data_transfer.files()
                && let Some(file) = files.get(0)
            {
                if let Some(handler) = on_drop_file.try_get_value() {
                    handler(file);
                }
            }
        }
    }

    pub fn call_on_drop_file(self, file: web_sys::File) {
        if let Some(handler) = self.on_drop_file.try_get_value() {
            handler(file);
        }
    }
}

pub fn use_drag_and_drop(on_drop_file: Arc<dyn Fn(web_sys::File) + Send + Sync>) -> DragDropState {
    DragDropState {
        is_drag_over: RwSignal::new(false),
        on_drop_file: StoredValue::new(on_drop_file),
    }
}
