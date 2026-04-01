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
    #[prop(optional, into)] test_id: Signal<String>,
) -> impl IntoView {
    let test_id_val = move || {
        let val = test_id.get();
        if val.is_empty() {
            None
        } else {
            Some(val)
        }
    };

    let step_test_id = move |idx: usize| {
        let base = test_id.get();
        if base.is_empty() {
            None
        } else {
            Some(format!("{}-step-{}", base, idx))
        }
    };

    view! {
        <div class="stepper" data-testid=test_id_val>
            <For
                each=move || steps.get().into_iter().enumerate()
                key=|(idx, _)| *idx
                children=move |(idx, step)| {
                    let step_number = step.number;
                    let step_label = step.label.clone();
                    let steps_len = steps.get().len();
                    let step_tst_id = move || step_test_id(idx);

                    view! {
                        <>
                            <div
                                class=move || {
                                    let active_idx = active.get();
                                    if idx < active_idx {
                                        "stepper-step completed"
                                    } else if idx == active_idx {
                                        "stepper-step active"
                                    } else {
                                        "stepper-step"
                                    }
                                }
                                data-testid=step_tst_id
                            >
                                <div class="stepper-number">{step_number}</div>
                                <span class="stepper-label stepper-label-desktop">{step_label.clone()}</span>
                            </div>
                            <Show when=move || idx < steps_len - 1>
                                <div class=move || {
                                    let active_idx = active.get();
                                    if idx < active_idx {
                                        "stepper-line"
                                    } else {
                                        "stepper-line stepper-line-inactive"
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
