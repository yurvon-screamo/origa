#[allow(dead_code)]
mod accordion;
mod alert;
#[allow(dead_code)]
mod app_skeleton;
mod audio_buttons;
mod audio_player;
#[allow(dead_code)]
mod avatar;
mod badge;
mod bottom_nav;
#[allow(dead_code)]
mod breadcrumbs;
#[allow(dead_code)]
mod button;
mod card;
mod card_action_bar;
mod card_history_modal;
mod chart;
mod checkbox;
mod collapsible;
mod connectivity_banner;
mod delete_button;
mod delete_confirm_modal;
#[allow(dead_code)]
mod divider;
mod drawer;
mod dropdown;
mod error_alert;
mod favorite_button;
mod furigana;
mod history_button;
mod input;
mod kanji_animation;
mod kanji_drawing;
#[allow(dead_code)]
mod label_frame;
#[allow(dead_code)]
mod layout;
mod level_selector;
mod loading;
mod logo;
mod markdown;
mod modal;
mod nav_config;
#[allow(dead_code)]
mod ocr_loading_stage;
mod page_header;
#[allow(dead_code)]
mod pagination;
mod progress;
#[allow(dead_code)]
mod radio;
mod reading_group;
mod search;
mod selected_count;
mod sidebar;
mod skeleton;

mod filter_tag;
#[allow(dead_code)]
mod stepper;
#[allow(dead_code)]
mod tab_button;
#[allow(dead_code)]
mod table;
mod tabs;
mod tag;
mod test_id;
mod text_to_speech;
#[allow(dead_code)]
mod toast;
#[allow(dead_code)]
mod toggle;
mod tooltip;
mod translator;
#[allow(dead_code)]
mod typography;
mod update_drawer;
mod word_audio;

pub use alert::{Alert, AlertType};
#[allow(unused_imports)]
pub use app_skeleton::AppSkeleton;
pub use audio_buttons::AudioButtons;
pub use audio_player::AudioPlayer;
pub use avatar::Avatar;
#[allow(unused_imports)]
pub use badge::Badge;
pub use bottom_nav::BottomTabBar;
#[allow(unused_imports)]
pub use button::{Button, ButtonSize, ButtonVariant};
pub use card::Card;
#[allow(unused_imports)]
pub use card_action_bar::CardActionBar;
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
pub use error_alert::ErrorAlert;
pub use favorite_button::FavoriteButton;
pub use furigana::FuriganaText;
pub use history_button::HistoryButton;
pub use input::Input;
pub use kanji_animation::{KanjiViewMode, KanjiWritingSection};
pub use kanji_drawing::KanjiDrawingPractice;
#[allow(unused_imports)]
pub use label_frame::LabelFrame;
pub use layout::{CardLayout, CardLayoutSize, PageLayout, PageLayoutVariant};
pub use level_selector::LevelSelector;
pub use loading::{LoadingOverlay, Spinner};
pub use logo::{Logo, LogoSize};
pub use markdown::{MarkdownText, MarkdownVariant};
pub use modal::Modal;
#[expect(unused_imports, reason = "заготовка для будущей навигации")]
pub use nav_config::NavRoute;
pub use ocr_loading_stage::{
    LoadingStageItem, OcrLoadingStage, OcrLoadingState, ProgressInfo, StageType, get_stage_info,
};
pub use page_header::PageHeader;
pub use progress::ProgressBar;
pub use reading_group::ReadingGroup;
pub use search::Search;
pub use selected_count::SelectedCount;
pub use sidebar::Sidebar;
pub use skeleton::Skeleton;
pub use translator::TranslatorText;

pub use filter_tag::FilterTag;
pub use stepper::{Stepper, StepperStep};
pub use tabs::{TabItem, Tabs};
pub use tag::{Tag, TagVariant};
pub use test_id::derive_test_id;
pub use text_to_speech::{
    get_reading_from_text, is_speech_supported, speak_text, speak_text_with_callback, stop_speech,
};
pub use toast::{ToastContainer, ToastData, ToastType};
pub use tooltip::Tooltip;
pub use typography::{DisplayText, Heading, HeadingLevel, Text, TextSize, TypographyVariant};
pub use update_drawer::UpdateDrawer;
pub use word_audio::{speak_word, speak_word_with_callback};
