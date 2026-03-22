use leptos::prelude::*;
use wasm_bindgen::closure::Closure;
use web_sys::{js_sys::Function, window};

#[derive(Clone)]
pub struct ConnectivityStore {
    pub is_online: RwSignal<bool>,
}

impl ConnectivityStore {
    pub fn new() -> Self {
        let is_online = window().map(|w| w.navigator().on_line()).unwrap_or(true);

        let is_online = RwSignal::new(is_online);

        if let Some(window) = window() {
            let window_online = window.clone();
            let is_online_online = is_online;
            let stored_closure = StoredValue::new_local(None::<Closure<dyn Fn(web_sys::Event)>>);
            let stored_closure_ptr = StoredValue::new_local(None::<Function>);

            Effect::new(move |_| {
                let w = window_online.clone();
                let closure = Closure::wrap(Box::new(move |_: web_sys::Event| {
                    is_online_online.set(true);
                }) as Box<dyn Fn(_)>);

                let closure_ptr = Function::from(closure.as_ref().clone());
                w.add_event_listener_with_callback("online", &closure_ptr)
                    .ok();
                stored_closure.set_value(Some(closure));
                stored_closure_ptr.set_value(Some(closure_ptr));

                on_cleanup(move || {
                    let closure_ptr = stored_closure_ptr.get_value().unwrap();
                    w.remove_event_listener_with_callback("online", &closure_ptr)
                        .ok();
                    stored_closure.set_value(None);
                    stored_closure_ptr.set_value(None);
                });
            });

            let window_offline = window.clone();
            let is_online_offline = is_online;
            let stored_closure = StoredValue::new_local(None::<Closure<dyn Fn(web_sys::Event)>>);
            let stored_closure_ptr = StoredValue::new_local(None::<Function>);

            Effect::new(move |_| {
                let w = window_offline.clone();
                let closure = Closure::wrap(Box::new(move |_: web_sys::Event| {
                    is_online_offline.set(false);
                }) as Box<dyn Fn(_)>);

                let closure_ptr = Function::from(closure.as_ref().clone());
                w.add_event_listener_with_callback("offline", &closure_ptr)
                    .ok();
                stored_closure.set_value(Some(closure));
                stored_closure_ptr.set_value(Some(closure_ptr));

                on_cleanup(move || {
                    let closure_ptr = stored_closure_ptr.get_value().unwrap();
                    w.remove_event_listener_with_callback("offline", &closure_ptr)
                        .ok();
                    stored_closure.set_value(None);
                    stored_closure_ptr.set_value(None);
                });
            });
        }

        Self { is_online }
    }
}

impl Default for ConnectivityStore {
    fn default() -> Self {
        Self::new()
    }
}
