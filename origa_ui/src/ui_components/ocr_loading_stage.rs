use crate::i18n::{Locale, use_i18n};
use leptos::prelude::*;
use leptos_i18n::I18nContext;

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
    #[prop(optional, into)] test_id: Signal<String>,
    status: StageStatus,
    title: String,
    description: String,
    #[prop(default = None)] progress: Option<ProgressInfo>,
    #[prop(default = None)] error_message: Option<String>,
) -> impl IntoView {
    let i18n = use_i18n();
    let test_id_val = move || {
        let val = test_id.get();
        if val.is_empty() { None } else { Some(val) }
    };
    let icon_class = match status {
        StageStatus::Waiting => "ocr-stage-icon-pending",
        StageStatus::Active => "ocr-stage-icon-processing",
        StageStatus::Completed => "ocr-stage-icon-success",
        StageStatus::Error => "ocr-stage-icon-error",
    };

    let icon_content = match status {
        StageStatus::Waiting => "\u{25CB}",
        StageStatus::Active => "\u{25C9}",
        StageStatus::Completed => "\u{2713}",
        StageStatus::Error => "\u{2717}",
    };

    let icon_label = move || match status {
        StageStatus::Waiting => i18n.get_keys().ui().ocr().waiting().inner().to_string(),
        StageStatus::Active => i18n.get_keys().ui().ocr().loading().inner().to_string(),
        StageStatus::Completed => i18n.get_keys().ui().ocr().completed().inner().to_string(),
        StageStatus::Error => i18n.get_keys().ui().ocr().error().inner().to_string(),
    };

    let card_class = match status {
        StageStatus::Active => "ocr-stage-bg-warm border ocr-stage-border-processing",
        StageStatus::Error => "ocr-stage-bg-warm border ocr-stage-border-error",
        StageStatus::Completed => "bg-[var(--bg-paper)] border ocr-stage-border-pending",
        StageStatus::Waiting => "bg-[var(--bg-aged)] border ocr-stage-border-pending",
    };

    let text_class = match status {
        StageStatus::Waiting => "ocr-stage-text-pending",
        _ => "text-[var(--fg-black)]",
    };

    let progress_view = move || {
        progress.and_then(|p| {
            if status == StageStatus::Active {
                let percent = p.percent.min(100);
                let loaded_mb = p.loaded_bytes as f64 / 1_048_576.0;
                let total_mb = p.total_bytes as f64 / 1_048_576.0;

                let details = if p.speed_bps > 0 {
                    let speed_mbps = p.speed_bps as f64 / 1_048_576.0;
                    if p.eta_seconds > 0 {
                        let eta_str = i18n
                            .get_keys()
                            .ui()
                            .seconds_short()
                            .inner()
                            .to_string()
                            .replacen("{}", &p.eta_seconds.to_string(), 1);
                        format!(
                            "{:.0} MB / {:.0} MB \u{2022} {:.1} MB/s \u{2022} {}",
                            loaded_mb, total_mb, speed_mbps, eta_str
                        )
                    } else {
                        format!(
                            "{:.0} MB / {:.0} MB \u{2022} {:.1} MB/s",
                            loaded_mb, total_mb, speed_mbps
                        )
                    }
                } else {
                    format!(
                        "{:.0} MB / {:.0} MB \u{2022} {}%",
                        loaded_mb, total_mb, percent
                    )
                };

                Some(view! {
                    <div class="mt-2 space-y-1">
                        <div class="progress-track">
                            <div
                                class="progress-fill"
                                style=format!("--progress-width: {}%", percent)
                            ></div>
                        </div>
                        <div class="text-xs ocr-stage-text-pending">{details}</div>
                    </div>
                })
            } else {
                None
            }
        })
    };

    let error_view = if status == StageStatus::Error {
        error_message.map(|msg| {
            view! { <div class="mt-2 text-xs ocr-stage-text-error">{msg}</div> }
        })
    } else {
        None
    };

    view! {
        <div class=format!("p-3 {}", card_class) data-testid=test_id_val>
            <div class="flex items-start gap-3">
                <span
                    class=format!("w-5 h-5 flex-shrink-0 {}", icon_class)
                    role="img"
                    aria-label=icon_label
                >{icon_content}</span>
                <div class="flex-1 min-w-0">
                    <div class=format!("text-sm font-medium {}", text_class)>{title}</div>
                    <div class="text-xs ocr-stage-text-pending mt-0.5">{description}</div>
                    {progress_view}
                    {error_view}
                </div>
            </div>
        </div>
    }
}

#[derive(Clone, Copy)]
pub enum StageType {
    Deim,
    Parseq,
    Init,
    Recognize,
}

#[derive(Clone, PartialEq)]
pub struct StageInfo {
    pub status: StageStatus,
    pub progress: Option<ProgressInfo>,
    pub description: String,
    pub error_message: Option<String>,
}

pub fn get_stage_info(
    i18n: &I18nContext<Locale>,
    stage: &OcrLoadingStage,
    stage_type: StageType,
) -> StageInfo {
    match stage_type {
        StageType::Deim => match stage {
            OcrLoadingStage::DownloadingDeim { progress } => StageInfo {
                status: StageStatus::Active,
                progress: Some(*progress),
                description: i18n
                    .get_keys()
                    .ui()
                    .ocr()
                    .deim_loading()
                    .inner()
                    .to_string(),
                error_message: None,
            },
            OcrLoadingStage::DownloadingParseq { .. }
            | OcrLoadingStage::Initializing { .. }
            | OcrLoadingStage::Recognizing
            | OcrLoadingStage::Completed => StageInfo {
                status: StageStatus::Completed,
                progress: None,
                description: i18n.get_keys().ui().ocr().deim_loaded().inner().to_string(),
                error_message: None,
            },
            OcrLoadingStage::Error { stage: s, message } if s == "deim" => StageInfo {
                status: StageStatus::Error,
                progress: None,
                description: i18n.get_keys().ui().ocr().load_error().inner().to_string(),
                error_message: Some(message.clone()),
            },
            _ => StageInfo {
                status: StageStatus::Waiting,
                progress: None,
                description: i18n
                    .get_keys()
                    .ui()
                    .ocr()
                    .deim_loading()
                    .inner()
                    .to_string(),
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
                description: i18n
                    .get_keys()
                    .ui()
                    .ocr()
                    .parseq_model()
                    .inner()
                    .to_string()
                    .replacen("{}", &current_model.to_string(), 1),
                error_message: None,
            },
            OcrLoadingStage::Initializing { .. }
            | OcrLoadingStage::Recognizing
            | OcrLoadingStage::Completed => StageInfo {
                status: StageStatus::Completed,
                progress: None,
                description: i18n
                    .get_keys()
                    .ui()
                    .ocr()
                    .parseq_loaded()
                    .inner()
                    .to_string(),
                error_message: None,
            },
            OcrLoadingStage::Error { stage: s, message } if s.starts_with("parseq") => StageInfo {
                status: StageStatus::Error,
                progress: None,
                description: i18n.get_keys().ui().ocr().load_error().inner().to_string(),
                error_message: Some(message.clone()),
            },
            _ => StageInfo {
                status: StageStatus::Waiting,
                progress: None,
                description: i18n
                    .get_keys()
                    .ui()
                    .ocr()
                    .parseq_loading()
                    .inner()
                    .to_string(),
                error_message: None,
            },
        },
        StageType::Init => match stage {
            OcrLoadingStage::Initializing { model_name } => StageInfo {
                status: StageStatus::Active,
                progress: None,
                description: i18n
                    .get_keys()
                    .ui()
                    .ocr()
                    .init_loading()
                    .inner()
                    .to_string()
                    .replacen("{}", model_name, 1),
                error_message: None,
            },
            OcrLoadingStage::Recognizing | OcrLoadingStage::Completed => StageInfo {
                status: StageStatus::Completed,
                progress: None,
                description: i18n
                    .get_keys()
                    .ui()
                    .ocr()
                    .models_loaded()
                    .inner()
                    .to_string(),
                error_message: None,
            },
            OcrLoadingStage::Error { stage: s, message } if s == "init" => StageInfo {
                status: StageStatus::Error,
                progress: None,
                description: i18n.get_keys().ui().ocr().init_error().inner().to_string(),
                error_message: Some(message.clone()),
            },
            _ => StageInfo {
                status: StageStatus::Waiting,
                progress: None,
                description: i18n
                    .get_keys()
                    .ui()
                    .ocr()
                    .waiting_status()
                    .inner()
                    .to_string(),
                error_message: None,
            },
        },
        StageType::Recognize => match stage {
            OcrLoadingStage::Recognizing => StageInfo {
                status: StageStatus::Active,
                progress: None,
                description: i18n
                    .get_keys()
                    .ui()
                    .ocr()
                    .processing_image()
                    .inner()
                    .to_string(),
                error_message: None,
            },
            OcrLoadingStage::Completed => StageInfo {
                status: StageStatus::Completed,
                progress: None,
                description: i18n
                    .get_keys()
                    .ui()
                    .ocr()
                    .completed_status()
                    .inner()
                    .to_string(),
                error_message: None,
            },
            OcrLoadingStage::Error { stage: s, message } if s == "recognize" => StageInfo {
                status: StageStatus::Error,
                progress: None,
                description: i18n
                    .get_keys()
                    .ui()
                    .ocr()
                    .recognition_error()
                    .inner()
                    .to_string(),
                error_message: Some(message.clone()),
            },
            _ => StageInfo {
                status: StageStatus::Waiting,
                progress: None,
                description: i18n
                    .get_keys()
                    .ui()
                    .ocr()
                    .waiting_status()
                    .inner()
                    .to_string(),
                error_message: None,
            },
        },
    }
}
