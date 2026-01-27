use leptos::prelude::*;

#[component]
pub fn Input(
    #[prop(into, optional)] label: Option<String>,
    #[prop(into, optional)] placeholder: Option<String>,
    #[prop(into, optional)] value: Option<Signal<String>>,
    #[prop(into, optional)] on_change: Option<Callback<String>>,
    #[prop(into, optional)] on_input: Option<Callback<String>>,
    #[prop(into, optional)] input_type: Option<InputType>,
    #[prop(into, optional)] error: Option<String>,
    #[prop(into, optional)] disabled: Option<bool>,
    #[prop(into, optional)] required: Option<bool>,
    #[prop(into, optional)] maxlength: Option<u32>,
    #[prop(into, optional)] multiline: Option<bool>,
    #[prop(into, optional)] rows: Option<u32>,
) -> impl IntoView {
    let (input_value, set_input_value) = value
        .map(|s| {
            // Create a local signal that syncs with the provided signal
            let (read, write) = signal(s.get());
            (read, write)
        })
        .unwrap_or_else(|| signal(String::new()));
    let input_type = input_type.unwrap_or(InputType::Text);
    let is_disabled = disabled.unwrap_or(false);
    let is_required = required.unwrap_or(false);
    let is_multiline = multiline.unwrap_or(false);
    let error_msg = error;

    let handle_input = move |ev| {
        let new_value = event_target_value(&ev);
        set_input_value.set(new_value.clone());
        if let Some(handler) = on_input {
            handler.run(new_value.clone());
        }
        if let Some(handler) = on_change {
            handler.run(new_value);
        }
    };

    let error_msg_clone = error_msg.clone();
    let has_error = Signal::derive(move || error_msg_clone.is_some());

    let input_type_str = match input_type {
        InputType::Text => "text",
        InputType::Email => "email",
        InputType::Password => "password",
        InputType::Number => "number",
        InputType::Tel => "tel",
        InputType::Url => "url",
    };
    let placeholder_val = placeholder.unwrap_or_default();
    let rows_val = rows.unwrap_or(4);

    view! {
        <div class="input-group">
            {label
                .map(|lbl| {
                    view! {
                        <label class="input-label">
                            {lbl}
                            {is_required.then(|| view! { <span class="input-required">*</span> })}
                        </label>
                    }
                })}
            {(!is_multiline)
                .then(|| {
                    view! {
                        <div>
                            <input
                                class=move || {
                                    format!(
                                        "input {}",
                                        if has_error.get() { "input-error" } else { "" },
                                    )
                                }
                                type=input_type_str
                                placeholder=placeholder_val.clone()
                                prop:value=move || input_value.get()
                                on:input=handle_input
                                disabled=is_disabled
                                required=is_required
                                maxlength=maxlength
                            />
                        </div>
                    }
                })}
            {is_multiline
                .then(|| {
                    view! {
                        <div>
                            <textarea
                                class=move || {
                                    format!(
                                        "input {}",
                                        if has_error.get() { "input-error" } else { "" },
                                    )
                                }
                                placeholder=placeholder_val.clone()
                                prop:value=move || input_value.get()
                                on:input=handle_input
                                disabled=is_disabled
                                maxlength=maxlength
                                rows=rows_val
                            />
                        </div>
                    }
                })} {error_msg.map(|err| view! { <div class="input-error">{err}</div> })}
        </div>
    }
}

#[derive(Clone, Copy, PartialEq, Default)]
pub enum InputType {
    #[default]
    Text,
    Email,
    Password,
    Number,
    Tel,
    Url,
}
