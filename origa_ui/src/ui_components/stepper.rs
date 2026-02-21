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
                each=move || {
                    let steps = steps.get();
                    let active_idx = active.get();
                    steps.into_iter().enumerate().map(move |(idx, step)| {
                        let status = if idx < active_idx {
                            StepStatus::Completed
                        } else if idx == active_idx {
                            StepStatus::Active
                        } else {
                            StepStatus::Pending
                        };
                        (idx, step, status)
                    }).collect::<Vec<_>>()
                }
                key=|(idx, _, _)| *idx
                children=move |(idx, step, status)| {
                    let steps_count = steps.get().len();
                    let is_last = idx == steps_count - 1;
                    let status_class = match status {
                        StepStatus::Active => "active",
                        StepStatus::Completed => "completed",
                        StepStatus::Pending => "",
                    };
                    let line_class = if matches!(status, StepStatus::Completed) { "" } else { "opacity-30" };
                    let step_number = step.number;
                    let step_label = step.label.clone();

                    view! {
                        <>
                            <div class=move || format!("stepper-step {}", status_class)>
                                <div class="stepper-number">{step_number}</div>
                                <span class="stepper-label hidden md:block">{step_label.clone()}</span>
                            </div>
                            <Show
                                when=move || !is_last
                                fallback=move || view! { <div class="stepper-line"></div> }
                            >
                                <div class=move || format!("stepper-line {}", line_class)></div>
                            </Show>
                        </>
                    }
                }
            />
        </div>
    }
}
