use leptos::prelude::*;

#[derive(Clone, Debug)]
pub struct StepperStep {
    pub number: u32,
    pub label: String,
}

#[derive(Clone, Copy, PartialEq, Default, Debug)]
pub enum StepStatus {
    #[default]
    Pending,
    Active,
    Completed,
}

#[component]
pub fn Stepper(
    #[prop(optional, into)] steps: Signal<Vec<StepperStep>>,
    #[prop(optional)] active: RwSignal<usize>,
) -> impl IntoView {
    view! {
        <div class="stepper">
            <For
                each=move || steps.get().into_iter().enumerate()
                key=|(idx, _)| *idx
                children=move |(idx, step)| {
                    let step_number = step.number;
                    let step_label = step.label.clone();
                    let steps_len = steps.get().len();

                    view! {
                        <>
                            <div class=move || {
                                let active_idx = active.get();
                                if idx < active_idx {
                                    "stepper-step completed"
                                } else if idx == active_idx {
                                    "stepper-step active"
                                } else {
                                    "stepper-step"
                                }
                            }>
                                <div class="stepper-number">{step_number}</div>
                                <span class="stepper-label hidden md:block">{step_label.clone()}</span>
                            </div>
                            <Show when=move || idx < steps_len - 1>
                                <div class=move || {
                                    let active_idx = active.get();
                                    if idx < active_idx {
                                        "stepper-line"
                                    } else {
                                        "stepper-line opacity-30"
                                    }
                                }></div>
                            </Show>
                        </>
                    }
                }
            />
        </div>
    }
}
