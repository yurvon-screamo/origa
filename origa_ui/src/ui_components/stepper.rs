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
    #[prop(into)] steps: Vec<StepperStep>,
    #[prop(optional)] active: RwSignal<usize>,
) -> impl IntoView {
    let steps_count = steps.len();

    view! {
        <div class="stepper">
            <For
                each=move || {
                    let steps = steps.clone();
                    let active_idx = active.get();
                    steps.iter().enumerate().map(move |(idx, step)| {
                        let status = if idx < active_idx {
                            StepStatus::Completed
                        } else if idx == active_idx {
                            StepStatus::Active
                        } else {
                            StepStatus::Pending
                        };
                        (idx, step.clone(), status)
                    }).collect::<Vec<_>>()
                }
                key=|(idx, _, _)| *idx
                children=move |(idx, step, status)| {
                    let is_last = idx == steps_count - 1;
                    let status_class = match status {
                        StepStatus::Active => "active",
                        StepStatus::Completed => "completed",
                        StepStatus::Pending => "",
                    };
                    let line_class = if matches!(status, StepStatus::Completed) { "" } else { "opacity-30" };

                    view! {
                        <>
                            <div class=format!("stepper-step {}", status_class)>
                                <div class="stepper-number">{step.number}</div>
                                <span class="stepper-label hidden md:block">{step.label}</span>
                            </div>
                            <Show
                                when=move || !is_last
                                fallback=move || view! { <div class="stepper-line"></div> }
                            >
                                <div class=format!("stepper-line {}", line_class)></div>
                            </Show>
                        </>
                    }
                }
            />
        </div>
    }
}
