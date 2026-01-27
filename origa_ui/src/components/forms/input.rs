use leptos::prelude::*;

#[component]
pub function Input(
    #[prop(into, optional)] label: Option<String>,
    #[prop(into, optional)] placeholder: Option<String>,
    #[prop(into, optional)] value: Option<Signal<String>>,
    #[prop(into, optional)] on_change: Option<Callback<String>>,
    #[prop(into, optional)] on_input: Option<Callback<String>>,
    #[prop(into, optional)] type: Option<InputType>,
    #[prop(into, optional)] error: Option<String>,
    #[prop(into, optional)] disabled: Option<bool>,
    #[prop(into, optional)] required: Option<bool>,
    #[prop(into, optional)] maxlength: Option<u32>,
    #[prop(into, optional)] multiline: Option<bool>,
) -> impl IntoView {
    let input_value = value.unwrap_or_else(|| create_signal("".to_string()));
    let input_type = type.unwrap_or(InputType::Text);
    let is_disabled = disabled.unwrap_or(false);
    let is_required = required.unwrap_or(false);
    let is_multiline = multiline.unwrap_or(false);
    let error_msg = error;
    
    let handle_input = move |ev| {
        let new_value = event_target_value(&ev);
        if let Some(handler) = on_input {
            handler.run(new_value.clone());
        }
        if let Some(handler) = on_change {
            handler.run(new_value);
        }
    };
    
    let has_error = Signal::derive(move || error_msg.as_ref().map(|_| true).unwrap_or(false));
    
    view! {
        <div class="input-group">
            <Show when=move || label.is_some()>
                <label class="input-label">
                    {move || label.clone().unwrap_or_default()}
                    {move || is_required.then(|| view! {
                        <span class="input-required">*</span>
                    })}
                </label>
            </Show>
            
            {move || if is_multiline {
                view! {
                    <textarea 
                        class=format!(
                            "input {}",
                            if has_error() { "input-error" } else { "" }
                        )
                        placeholder=placeholder.unwrap_or_default()
                        prop:value=input_value
                        on:input=handle_input
                        disabled=is_disabled
                        maxlength=maxlength
                        rows=4
                    />
                }
            } else {
                let input_type_str = match input_type {
                    InputType::Text => "text",
                    InputType::Email => "email",
                    InputType::Password => "password",
                    InputType::Number => "number",
                    InputType::Tel => "tel",
                    InputType::Url => "url",
                };
                
                view! {
                    <input 
                        class=format!(
                            "input {}",
                            if has_error() { "input-error" } else { "" }
                        )
                        type=input_type_str
                        placeholder=placeholder.unwrap_or_default()
                        prop:value=input_value
                        on:input=handle_input
                        disabled=is_disabled
                        required=is_required
                        maxlength=maxlength
                    />
                }
            }}
            
            <Show when=move || error_msg.is_some()>
                <div class="input-error">
                    {move || error_msg.clone().unwrap_or_default()}
                </div>
            </Show>
        </div>
    }
}

#[component]
pub function Textarea(
    #[prop(into, optional)] label: Option<String>,
    #[prop(into, optional)] placeholder: Option<String>,
    #[prop(into, optional)] value: Option<Signal<String>>,
    #[prop(into, optional)] on_change: Option<Callback<String>>,
    #[prop(into, optional)] error: Option<String>,
    #[prop(into, optional)] disabled: Option<bool>,
    #[prop(into, optional)] rows: Option<u32>,
) -> impl IntoView {
    Input(
        label=label,
        placeholder=placeholder,
        value=value,
        on_change=on_change,
        error=error,
        disabled=disabled,
        multiline=true,
        rows=rows.unwrap_or(4),
    )
}

#[derive(Clone, Copy, PartialEq)]
pub enum InputType {
    Text,
    Email,
    Password,
    Number,
    Tel,
    Url,
}

impl Default for InputType {
    fn default() -> Self {
        InputType::Text
    }
}