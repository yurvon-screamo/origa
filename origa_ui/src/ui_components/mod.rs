#![allow(dead_code)]

mod accordion;
mod alert;
mod app_skeleton;
mod audio_buttons;
mod avatar;
mod badge;
mod breadcrumbs;
mod button;
mod card;
mod card_history_modal;
mod chart;
mod checkbox;
mod collapsible;
mod connectivity_banner;
mod delete_button;
mod delete_confirm_modal;
mod divider;
mod drawer;
mod dropdown;
mod favorite_button;
mod footer;
mod furigana;
mod history_button;
mod input;
mod kanji_animation;
mod kanji_drawing;
mod label_frame;
mod layout;
mod loading;
mod markdown;
mod modal;
mod navbar;
mod ocr_loading_stage;
mod pagination;
mod progress;
mod radio;
mod reading_group;
mod search;
mod skeleton;

mod stepper;
mod tab_button;
mod table;
mod tabs;
mod tag;
mod text_to_speech;
mod toast;
mod toggle;
mod tooltip;
mod typography;
mod update_drawer;

pub use alert::{Alert, AlertType};
#[allow(unused_imports)]
pub use app_skeleton::AppSkeleton;
pub use audio_buttons::AudioButtons;
pub use avatar::Avatar;
#[allow(unused_imports)]
pub use badge::Badge;
pub use button::{Button, ButtonSize, ButtonVariant};
pub use card::Card;
pub use card_history_modal::CardHistoryModal;
pub use chart::LineChart;
pub use checkbox::Checkbox;
pub use collapsible::CollapsibleDescription;
pub use connectivity_banner::ConnectivityBanner;
pub use delete_button::DeleteButton;
pub use delete_confirm_modal::DeleteConfirmModal;
pub use divider::{Divider, DividerVariant};
pub use drawer::Drawer;
pub use dropdown::{Dropdown, DropdownItem};
pub use favorite_button::FavoriteButton;
pub use furigana::FuriganaText;
pub use history_button::HistoryButton;
pub use input::Input;
pub use kanji_animation::{KanjiViewMode, KanjiWritingSection};
pub use kanji_drawing::KanjiDrawingPractice;
pub use label_frame::LabelFrame;
pub use layout::{CardLayout, CardLayoutSize, PageLayout, PageLayoutVariant};
pub use loading::{LoadingOverlay, Spinner};
pub use markdown::{MarkdownText, MarkdownVariant};
pub use modal::Modal;
pub use ocr_loading_stage::{
    LoadingStageItem, OcrLoadingStage, OcrLoadingState, ProgressInfo, StageType, get_stage_info,
};
pub use progress::ProgressBar;
pub use reading_group::ReadingGroup;
pub use search::Search;
pub use skeleton::Skeleton;

pub use stepper::{Stepper, StepperStep};
pub use tabs::{TabItem, Tabs};
pub use tag::{Tag, TagVariant};
pub use text_to_speech::{
    get_reading_from_text, is_speech_supported, speak_text, speak_text_with_callback, stop_speech,
};
pub use toast::{ToastContainer, ToastData, ToastType};
pub use tooltip::Tooltip;
pub use typography::{DisplayText, Heading, HeadingLevel, Text, TextSize, TypographyVariant};
pub use update_drawer::UpdateDrawer;
