use leptos::prelude::*;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum StageStatus {
    Waiting,
    Active,
    Completed,
    Error,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ProgressInfo {
    pub percent: u32,
    pub loaded_bytes: u64,
    pub total_bytes: u64,
    pub speed_bps: u64,
    pub eta_seconds: u64,
}

#[derive(Clone, Debug, Default)]
pub enum OcrLoadingStage {
    #[default]
    Idle,
    DownloadingDeim {
        progress: ProgressInfo,
    },
    DownloadingParseq {
        current_model: usize,
        progress: ProgressInfo,
    },
    Initializing {
        model_name: String,
    },
    Recognizing,
    Completed,
    Error {
        stage: String,
        #[allow(dead_code)]
        message: String,
    },
}

#[derive(Clone)]
pub struct OcrLoadingState {
    pub stage: RwSignal<OcrLoadingStage>,
    pub cancel_requested: RwSignal<bool>,
    pub start_time: RwSignal<Option<f64>>,
}

impl OcrLoadingState {
    pub fn new() -> Self {
        Self {
            stage: RwSignal::new(OcrLoadingStage::Idle),
            cancel_requested: RwSignal::new(false),
            start_time: RwSignal::new(None),
        }
    }

    #[allow(dead_code)]
    pub fn is_loading(&self) -> bool {
        !matches!(
            self.stage.get(),
            OcrLoadingStage::Idle | OcrLoadingStage::Completed | OcrLoadingStage::Error { .. }
        )
    }

    pub fn reset(&self) {
        self.stage.set(OcrLoadingStage::Idle);
        self.cancel_requested.set(false);
        self.start_time.set(None);
    }
}

impl Default for OcrLoadingState {
    fn default() -> Self {
        Self::new()
    }
}

#[component]
pub fn LoadingStageItem(
    status: StageStatus,
    title: String,
    description: String,
    #[prop(default = None)] progress: Option<ProgressInfo>,
    #[prop(default = None)] error_message: Option<String>,
) -> impl IntoView {
    let (icon_class, icon_content) = match status {
        StageStatus::Waiting => ("text-slate-600", "\u{25CB}"),
        StageStatus::Active => ("text-sky-500 animate-spin", "\u{25C9}"),
        StageStatus::Completed => ("text-emerald-500", "\u{2713}"),
        StageStatus::Error => ("text-rose-500", "\u{2717}"),
    };

    let card_class = match status {
        StageStatus::Active => "bg-slate-800 border border-sky-500/30 ring-1 ring-sky-500/20",
        StageStatus::Error => "bg-rose-500/5 border border-rose-500/30",
        StageStatus::Completed => "bg-slate-800/30 border border-slate-700/50",
        StageStatus::Waiting => "bg-slate-800/50 border border-slate-700",
    };

    let text_class = match status {
        StageStatus::Waiting => "text-slate-500",
        _ => "text-slate-200",
    };

    let progress_view = progress.and_then(|p| {
        if status == StageStatus::Active {
            let percent = p.percent.min(100);
            let loaded_mb = p.loaded_bytes as f64 / 1_048_576.0;
            let total_mb = p.total_bytes as f64 / 1_048_576.0;

            let details = if p.speed_bps > 0 {
                let speed_mbps = p.speed_bps as f64 / 1_048_576.0;
                if p.eta_seconds > 0 {
                    format!(
                        "{:.0} MB / {:.0} MB \u{2022} {:.1} MB/s \u{2022} ~{} \u{441}\u{435}\u{43a}",
                        loaded_mb, total_mb, speed_mbps, p.eta_seconds
                    )
                } else {
                    format!(
                        "{:.0} MB / {:.0} MB \u{2022} {:.1} MB/s",
                        loaded_mb, total_mb, speed_mbps
                    )
                }
            } else {
                format!("{:.0} MB / {:.0} MB \u{2022} {}%", loaded_mb, total_mb, percent)
            };

            Some(view! {
                <div class="mt-2 space-y-1">
                    <div class="w-full h-1 rounded-full bg-slate-700 overflow-hidden">
                        <div
                            class="h-full rounded-full bg-sky-500 transition-all duration-300 ease-out"
                            style=format!("width: {}%", percent)
                        ></div>
                    </div>
                    <div class="text-xs text-slate-400">{details}</div>
                </div>
            })
        } else {
            None
        }
    });

    let error_view = if status == StageStatus::Error {
        error_message.map(|msg| {
            view! { <div class="mt-2 text-xs text-rose-400">{msg}</div> }
        })
    } else {
        None
    };

    view! {
        <div class=format!("p-3 rounded-lg {}", card_class)>
            <div class="flex items-start gap-3">
                <span class=format!("w-5 h-5 flex-shrink-0 {}", icon_class)>{icon_content}</span>
                <div class="flex-1 min-w-0">
                    <div class=format!("text-sm font-medium {}", text_class)>{title}</div>
                    <div class="text-xs text-slate-400 mt-0.5">{description}</div>
                    {progress_view}
                    {error_view}
                </div>
            </div>
        </div>
    }
}

#[component]
pub fn OcrLoadingModal(
    state: OcrLoadingState,
    #[prop(optional, into)] on_cancel: Option<Callback<()>>,
    #[prop(optional, into)] on_retry: Option<Callback<()>>,
) -> impl IntoView {
    let stage = state.stage;
    let cancel_requested = state.cancel_requested;

    let deim_info = Memo::new(move |_| {
        match stage.get() {
            OcrLoadingStage::DownloadingDeim { progress } => {
                (StageStatus::Active, Some(progress), "Deim \u{2022} ~50 MB".to_string())
            }
            OcrLoadingStage::DownloadingParseq { .. }
            | OcrLoadingStage::Initializing { .. }
            | OcrLoadingStage::Recognizing
            | OcrLoadingStage::Completed => {
                (StageStatus::Completed, None, "Deim \u{2022} \u{437}\u{430}\u{433}\u{440}\u{443}\u{436}\u{435}\u{43d}\u{43e}".to_string())
            }
            OcrLoadingStage::Error { stage: s, .. } if s == "deim" => {
                (StageStatus::Error, None, "\u{41e}\u{448}\u{438}\u{431}\u{43a}\u{430} \u{437}\u{430}\u{433}\u{440}\u{443}\u{437}\u{43a}\u{438}".to_string())
            }
            _ => (StageStatus::Waiting, None, "Deim \u{2022} ~50 MB".to_string()),
        }
    });

    let parseq_info = Memo::new(move |_| {
        match stage.get() {
            OcrLoadingStage::DownloadingParseq { current_model, progress } => {
                let desc = format!("Parseq \u{2022} \u{43c}\u{43e}\u{434}\u{435}\u{43b}\u{44c} {}/3", current_model);
                (StageStatus::Active, Some(progress), desc)
            }
            OcrLoadingStage::Initializing { .. }
            | OcrLoadingStage::Recognizing
            | OcrLoadingStage::Completed => {
                (StageStatus::Completed, None, "Parseq \u{2022} \u{437}\u{430}\u{433}\u{440}\u{443}\u{436}\u{435}\u{43d}\u{43e}".to_string())
            }
            OcrLoadingStage::Error { stage: s, .. } if s.starts_with("parseq") => {
                (StageStatus::Error, None, "\u{41e}\u{448}\u{438}\u{431}\u{43a}\u{430} \u{437}\u{430}\u{433}\u{440}\u{443}\u{437}\u{43a}\u{438}".to_string())
            }
            _ => (StageStatus::Waiting, None, "Parseq \u{2022} ~100 MB".to_string()),
        }
    });

    let init_info = Memo::new(move |_| {
        match stage.get() {
            OcrLoadingStage::Initializing { model_name } => {
                let desc = format!("\u{417}\u{430}\u{433}\u{440}\u{443}\u{437}\u{43a}\u{430} {} \u{432} \u{43f}\u{430}\u{43c}\u{44f}\u{442}\u{44c}...", model_name);
                (StageStatus::Active, desc)
            }
            OcrLoadingStage::Recognizing | OcrLoadingStage::Completed => {
                (StageStatus::Completed, "\u{41c}\u{43e}\u{434}\u{435}\u{43b}\u{438} \u{437}\u{430}\u{433}\u{440}\u{443}\u{436}\u{435}\u{43d}\u{44b}".to_string())
            }
            OcrLoadingStage::Error { stage: s, .. } if s == "init" => {
                (StageStatus::Error, "\u{41e}\u{448}\u{438}\u{431}\u{43a}\u{430} \u{438}\u{43d}\u{438}\u{446}\u{438}\u{430}\u{43b}\u{438}\u{437}\u{430}\u{446}\u{438}\u{438}".to_string())
            }
            _ => (StageStatus::Waiting, "\u{41e}\u{436}\u{438}\u{434}\u{430}\u{43d}\u{438}\u{435}...".to_string()),
        }
    });

    let recognize_info = Memo::new(move |_| {
        match stage.get() {
            OcrLoadingStage::Recognizing => {
                (StageStatus::Active, "\u{41e}\u{431}\u{440}\u{430}\u{431}\u{43e}\u{442}\u{43a}\u{430} \u{438}\u{437}\u{43e}\u{431}\u{440}\u{430}\u{436}\u{435}\u{43d}\u{438}\u{44f}...".to_string())
            }
            OcrLoadingStage::Completed => {
                (StageStatus::Completed, "\u{417}\u{430}\u{432}\u{435}\u{440}\u{448}\u{435}\u{43d}\u{43e}".to_string())
            }
            OcrLoadingStage::Error { stage: s, .. } if s == "recognize" => {
                (StageStatus::Error, "\u{41e}\u{448}\u{438}\u{431}\u{43a}\u{430} \u{440}\u{430}\u{441}\u{43f}\u{43e}\u{437}\u{43d}\u{430}\u{432}\u{430}\u{43d}\u{438}\u{44f}".to_string())
            }
            _ => (StageStatus::Waiting, "\u{41e}\u{436}\u{438}\u{434}\u{430}\u{43d}\u{438}\u{435}...".to_string()),
        }
    });

    let is_error = Memo::new(move |_| matches!(stage.get(), OcrLoadingStage::Error { .. }));

    let handle_cancel = move || {
        cancel_requested.set(true);
        if let Some(cb) = on_cancel {
            cb.run(());
        }
    };

    let handle_retry = move || {
        if let Some(cb) = on_retry {
            cb.run(());
        }
    };

    let title_icon = move || {
        if is_error.get() {
            view! { <span class="text-rose-500">"\u{26a0}"</span> }.into_any()
        } else {
            view! { <span class="text-sky-500">"\u{25c9}"</span> }.into_any()
        }
    };

    let title_text = move || {
        if is_error.get() {
            "\u{41e}\u{448}\u{438}\u{431}\u{43a}\u{430} \u{437}\u{430}\u{433}\u{440}\u{443}\u{437}\u{43a}\u{438}"
        } else {
            "\u{41f}\u{43e}\u{434}\u{433}\u{43e}\u{442}\u{43e}\u{432}\u{43a}\u{430} \u{43a} \u{440}\u{430}\u{441}\u{43f}\u{43e}\u{437}\u{43d}\u{430}\u{432}\u{430}\u{43d}\u{438}\u{44e}"
        }
    };

    let buttons = move || {
        if is_error.get() {
            view! {
                <>
                    <button
                        class="px-4 py-2 rounded-lg bg-slate-700 hover:bg-slate-600 text-slate-200 font-medium transition-colors duration-150"
                        on:click=move |_| handle_cancel()
                    >
                        "\u{41e}\u{442}\u{43c}\u{435}\u{43d}\u{430}"
                    </button>
                    <button
                        class="px-4 py-2 rounded-lg bg-sky-500 hover:bg-sky-400 text-white font-medium transition-colors duration-150"
                        on:click=move |_| handle_retry()
                    >
                        "\u{41f}\u{43e}\u{432}\u{442}\u{43e}\u{440}\u{438}\u{442}\u{44c}"
                    </button>
                </>
            }
            .into_any()
        } else {
            view! {
                <button
                    class="px-4 py-2 rounded-lg bg-slate-700 hover:bg-slate-600 text-slate-200 font-medium transition-colors duration-150 disabled:opacity-50 disabled:cursor-not-allowed"
                    on:click=move |_| handle_cancel()
                    disabled=cancel_requested.get()
                >
                    {move || if cancel_requested.get() { "\u{41e}\u{442}\u{43c}\u{435}\u{43d}\u{430}..." } else { "\u{41e}\u{442}\u{43c}\u{435}\u{43d}\u{438}\u{442}\u{44c}" }}
                </button>
            }
            .into_any()
        }
    };

    view! {
        <div
            class="fixed inset-0 z-50 flex items-center justify-center bg-slate-900/80 backdrop-blur-sm"
            role="dialog"
            aria-modal="true"
            aria-labelledby="ocr-loading-title"
        >
            <div class="w-full max-w-md mx-4 bg-slate-900 rounded-xl shadow-2xl border border-slate-700 p-6 space-y-4">
                <h2
                    id="ocr-loading-title"
                    class="text-lg font-semibold text-slate-100 flex items-center gap-2"
                >
                    {title_icon}
                    {title_text}
                </h2>

                <div class="space-y-3" role="list">
                    <LoadingStageItem
                        status=deim_info.get().0
                        title="\u{421}\u{435}\u{433}\u{43c}\u{435}\u{43d}\u{442}\u{430}\u{446}\u{438}\u{44f} \u{442}\u{435}\u{43a}\u{441}\u{442}\u{430}".to_string()
                        description=deim_info.get().2
                        progress=deim_info.get().1
                    />

                    <LoadingStageItem
                        status=parseq_info.get().0
                        title="\u{420}\u{430}\u{441}\u{43f}\u{43e}\u{437}\u{43d}\u{430}\u{432}\u{430}\u{43d}\u{438}\u{435} \u{441}\u{438}\u{43c}\u{432}\u{43e}\u{43b}\u{43e}\u{432}".to_string()
                        description=parseq_info.get().2
                        progress=parseq_info.get().1
                    />

                    <LoadingStageItem
                        status=init_info.get().0
                        title="\u{418}\u{43d}\u{438}\u{446}\u{438}\u{430}\u{43b}\u{438}\u{437}\u{430}\u{446}\u{438}\u{44f} \u{43c}\u{43e}\u{434}\u{435}\u{43b}\u{435}\u{439}".to_string()
                        description=init_info.get().1
                    />

                    <LoadingStageItem
                        status=recognize_info.get().0
                        title="\u{420}\u{430}\u{441}\u{43f}\u{43e}\u{437}\u{43d}\u{430}\u{432}\u{430}\u{43d}\u{438}\u{435} \u{442}\u{435}\u{43a}\u{441}\u{442}\u{430}".to_string()
                        description=recognize_info.get().1
                    />
                </div>

                <div class="flex justify-end gap-2 pt-2">
                    {buttons}
                </div>
            </div>
        </div>
    }
}
