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

impl ProgressInfo {
    #[allow(dead_code)]
    pub fn new(loaded_bytes: u64, total_bytes: u64, elapsed_secs: f64) -> Self {
        let percent = if total_bytes > 0 {
            (loaded_bytes as f64 / total_bytes as f64 * 100.0).min(100.0) as u32
        } else {
            0
        };

        let speed_bps = if elapsed_secs > 0.0 && loaded_bytes > 0 {
            (loaded_bytes as f64 / elapsed_secs) as u64
        } else {
            0
        };

        let eta_seconds = if speed_bps > 0 && total_bytes > loaded_bytes {
            (total_bytes - loaded_bytes) / speed_bps
        } else {
            0
        };

        Self {
            percent,
            loaded_bytes,
            total_bytes,
            speed_bps,
            eta_seconds,
        }
    }
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

#[derive(Clone, Copy)]
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
    let (icon_class, icon_content, icon_label) = match status {
        StageStatus::Waiting => ("text-slate-600", "\u{25CB}", "Ожидание"),
        StageStatus::Active => ("text-sky-500 animate-spin", "\u{25C9}", "Загрузка"),
        StageStatus::Completed => ("text-emerald-500", "\u{2713}", "Завершено"),
        StageStatus::Error => ("text-rose-500", "\u{2717}", "Ошибка"),
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
                        "{:.0} MB / {:.0} MB \u{2022} {:.1} MB/s \u{2022} ~{} сек",
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
                <span
                    class=format!("w-5 h-5 flex-shrink-0 {}", icon_class)
                    role="img"
                    aria-label=icon_label
                >{icon_content}</span>
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

enum StageType {
    Deim,
    Parseq,
    Init,
    Recognize,
}

#[derive(Clone, PartialEq)]
struct StageInfo {
    status: StageStatus,
    progress: Option<ProgressInfo>,
    description: String,
    error_message: Option<String>,
}

fn get_stage_info(stage: &OcrLoadingStage, stage_type: StageType) -> StageInfo {
    match stage_type {
        StageType::Deim => match stage {
            OcrLoadingStage::DownloadingDeim { progress } => StageInfo {
                status: StageStatus::Active,
                progress: Some(*progress),
                description: "Deim \u{2022} ~50 MB".into(),
                error_message: None,
            },
            OcrLoadingStage::DownloadingParseq { .. }
            | OcrLoadingStage::Initializing { .. }
            | OcrLoadingStage::Recognizing
            | OcrLoadingStage::Completed => StageInfo {
                status: StageStatus::Completed,
                progress: None,
                description: "Deim \u{2022} загружено".into(),
                error_message: None,
            },
            OcrLoadingStage::Error { stage: s, message } if s == "deim" => StageInfo {
                status: StageStatus::Error,
                progress: None,
                description: "Ошибка загрузки".into(),
                error_message: Some(message.clone()),
            },
            _ => StageInfo {
                status: StageStatus::Waiting,
                progress: None,
                description: "Deim \u{2022} ~50 MB".into(),
                error_message: None,
            },
        },
        StageType::Parseq => match stage {
            OcrLoadingStage::DownloadingParseq {
                current_model,
                progress,
            } => StageInfo {
                status: StageStatus::Active,
                progress: Some(*progress),
                description: format!("Parseq \u{2022} модель {}/3", current_model),
                error_message: None,
            },
            OcrLoadingStage::Initializing { .. }
            | OcrLoadingStage::Recognizing
            | OcrLoadingStage::Completed => StageInfo {
                status: StageStatus::Completed,
                progress: None,
                description: "Parseq \u{2022} загружено".into(),
                error_message: None,
            },
            OcrLoadingStage::Error { stage: s, message } if s.starts_with("parseq") => StageInfo {
                status: StageStatus::Error,
                progress: None,
                description: "Ошибка загрузки".into(),
                error_message: Some(message.clone()),
            },
            _ => StageInfo {
                status: StageStatus::Waiting,
                progress: None,
                description: "Parseq \u{2022} ~100 MB".into(),
                error_message: None,
            },
        },
        StageType::Init => match stage {
            OcrLoadingStage::Initializing { model_name } => StageInfo {
                status: StageStatus::Active,
                progress: None,
                description: format!("Загрузка {} в память...", model_name),
                error_message: None,
            },
            OcrLoadingStage::Recognizing | OcrLoadingStage::Completed => StageInfo {
                status: StageStatus::Completed,
                progress: None,
                description: "Модели загружены".into(),
                error_message: None,
            },
            OcrLoadingStage::Error { stage: s, message } if s == "init" => StageInfo {
                status: StageStatus::Error,
                progress: None,
                description: "Ошибка инициализации".into(),
                error_message: Some(message.clone()),
            },
            _ => StageInfo {
                status: StageStatus::Waiting,
                progress: None,
                description: "Ожидание...".into(),
                error_message: None,
            },
        },
        StageType::Recognize => match stage {
            OcrLoadingStage::Recognizing => StageInfo {
                status: StageStatus::Active,
                progress: None,
                description: "Обработка изображения...".into(),
                error_message: None,
            },
            OcrLoadingStage::Completed => StageInfo {
                status: StageStatus::Completed,
                progress: None,
                description: "Завершено".into(),
                error_message: None,
            },
            OcrLoadingStage::Error { stage: s, message } if s == "recognize" => StageInfo {
                status: StageStatus::Error,
                progress: None,
                description: "Ошибка распознавания".into(),
                error_message: Some(message.clone()),
            },
            _ => StageInfo {
                status: StageStatus::Waiting,
                progress: None,
                description: "Ожидание...".into(),
                error_message: None,
            },
        },
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

    let deim_info = Memo::new(move |_| get_stage_info(&stage.get(), StageType::Deim));
    let parseq_info = Memo::new(move |_| get_stage_info(&stage.get(), StageType::Parseq));
    let init_info = Memo::new(move |_| get_stage_info(&stage.get(), StageType::Init));
    let recognize_info = Memo::new(move |_| get_stage_info(&stage.get(), StageType::Recognize));

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

    let handle_keydown = move |ev: leptos::ev::KeyboardEvent| {
        if ev.key() == "Escape" {
            ev.prevent_default();
            handle_cancel();
        }
    };

    let title_icon = move || {
        if is_error.get() {
            view! {
                <span class="text-rose-500" role="img" aria-label="Предупреждение">
                    "\u{26a0}"
                </span>
            }
            .into_any()
        } else {
            view! {
                <span class="text-sky-500" role="img" aria-label="Загрузка">
                    "\u{25c9}"
                </span>
            }
            .into_any()
        }
    };

    let title_text = move || {
        if is_error.get() {
            "Ошибка загрузки"
        } else {
            "Подготовка к распознаванию"
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
                        "Отмена"
                    </button>
                    <button
                        class="px-4 py-2 rounded-lg bg-sky-500 hover:bg-sky-400 text-white font-medium transition-colors duration-150"
                        on:click=move |_| handle_retry()
                    >
                        "Повторить"
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
                    {move || if cancel_requested.get() { "Отмена..." } else { "Отменить" }}
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
            aria-describedby="ocr-loading-desc"
            tabindex="-1"
            on:keydown=handle_keydown
        >
            <div class="w-full max-w-md mx-4 bg-slate-900 rounded-xl shadow-2xl border border-slate-700 p-6 space-y-4">
                <h2
                    id="ocr-loading-title"
                    class="text-lg font-semibold text-slate-100 flex items-center gap-2"
                >
                    {title_icon}
                    {title_text}
                </h2>

                <div id="ocr-loading-desc" class="sr-only">
                    "Загрузка моделей для распознавания японского текста"
                </div>

                <div class="space-y-3" role="list">
                    <LoadingStageItem
                        status=deim_info.get().status
                        title="Сегментация текста".to_string()
                        description=deim_info.get().description
                        progress=deim_info.get().progress
                        error_message=deim_info.get().error_message
                    />

                    <LoadingStageItem
                        status=parseq_info.get().status
                        title="Распознавание символов".to_string()
                        description=parseq_info.get().description
                        progress=parseq_info.get().progress
                        error_message=parseq_info.get().error_message
                    />

                    <LoadingStageItem
                        status=init_info.get().status
                        title="Инициализация моделей".to_string()
                        description=init_info.get().description
                        progress=init_info.get().progress
                        error_message=init_info.get().error_message
                    />

                    <LoadingStageItem
                        status=recognize_info.get().status
                        title="Распознавание текста".to_string()
                        description=recognize_info.get().description
                        progress=recognize_info.get().progress
                        error_message=recognize_info.get().error_message
                    />
                </div>

                <div class="flex justify-end gap-2 pt-2">
                    {buttons}
                </div>
            </div>
        </div>
    }
}
