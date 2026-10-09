//! Rendering of the inbox processing states inside the add-words drawer.
//!
//! The view reuses the existing progress primitives — the OCR stepper from
//! the image stage and the audio status line — so a zero-tap run shows the
//! same phase detail (model download, recognition) as the manual tabs.

use super::{AudioState, InboxSignals, OcrState, stage_item_view};
use crate::i18n::{t, use_i18n};
use crate::ui_components::{
    Alert, AlertType, Button, ButtonVariant, StageType, Text, TextSize, TypographyVariant,
};
use leptos::prelude::*;

/// Which processing surface the inbox should render.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::pages::words) enum InboxViewStage {
    OcrProgress,
    AudioProgress,
    Failed,
    GenericProgress,
}

/// Pure resolution of the visible inbox surface. The priority order is
/// OCR in-flight > audio in-flight > failure > generic waiting.
pub(in crate::pages::words) fn resolve_view_stage(
    ocr_state: OcrState,
    audio_state: AudioState,
    has_error: bool,
) -> InboxViewStage {
    if ocr_state == OcrState::Processing {
        InboxViewStage::OcrProgress
    } else if matches!(
        audio_state,
        AudioState::LoadingModel | AudioState::Processing
    ) {
        InboxViewStage::AudioProgress
    } else if has_error {
        InboxViewStage::Failed
    } else {
        InboxViewStage::GenericProgress
    }
}

#[component]
pub(in crate::pages::words) fn InboxStageView(
    inbox: InboxSignals,
    on_cancel: Callback<()>,
    on_open_manually: Callback<()>,
) -> impl IntoView {
    let i18n = use_i18n();

    // Tracked reads so the surface switches as the pipeline progresses.
    let current_stage = move || {
        resolve_view_stage(
            inbox.ocr_state.get(),
            inbox.audio_state.get(),
            inbox.error.get().is_some(),
        )
    };

    view! {
        {move || match current_stage() {
            InboxViewStage::OcrProgress => {
                let stage = inbox.ocr_loading_state.stage;
                view! {
                    <div class="space-y-4" data-testid="words-inbox-ocr-progress">
                        <div class="space-y-3" role="list">
                            {stage_item_view(&i18n, stage, StageType::Deim, i18n.get_keys().words().image().segmentation().inner().to_string())}
                            {stage_item_view(&i18n, stage, StageType::Parseq, i18n.get_keys().words().image().recognition().inner().to_string())}
                            {stage_item_view(&i18n, stage, StageType::Init, i18n.get_keys().words().image().initialization().inner().to_string())}
                            {stage_item_view(&i18n, stage, StageType::Recognize, i18n.get_keys().words().image().text_recognition().inner().to_string())}
                        </div>
                        <InboxCancelButton on_cancel=on_cancel />
                    </div>
                }
                .into_any()
            },
            InboxViewStage::AudioProgress => view! {
                <div class="space-y-4" data-testid="words-inbox-processing">
                    <div class="text-lg font-semibold text-[var(--fg-black)] flex items-center gap-2">
                        <span class="spinner spinner-sm"></span>
                        {move || {
                            inbox
                                .audio_status_text
                                .get()
                                .unwrap_or_else(|| {
                                    i18n.get_keys().words().inbox().transcribing_audio().inner().to_string()
                                })
                        }}
                    </div>
                    <InboxCancelButton on_cancel=on_cancel />
                </div>
            }
            .into_any(),
            InboxViewStage::Failed => view! {
                <div class="space-y-4" data-testid="words-inbox-error">
                    <Alert
                        alert_type=Signal::derive(|| AlertType::Warning)
                        title=Signal::derive(move || {
                            i18n.get_keys().words().inbox().processing_failed().inner().to_string()
                        })
                        message=Signal::derive(move || {
                            inbox.error.get().unwrap_or_default()
                        })
                    />
                    <Button
                        variant=ButtonVariant::Ghost
                        on_click=Callback::new(move |_| on_open_manually.run(()))
                        test_id="words-inbox-open-manually-btn"
                    >
                        {t!(i18n, words.inbox.open_manually)}
                    </Button>
                </div>
            }
            .into_any(),
            InboxViewStage::GenericProgress => view! {
                <div class="space-y-4" data-testid="words-inbox-processing">
                    <div class="flex items-center justify-center py-8 gap-3">
                        <span class="spinner spinner-sm"></span>
                        <Text size=TextSize::Default variant=TypographyVariant::Muted>
                            {t!(i18n, words.inbox.processing)}
                        </Text>
                    </div>
                    <InboxCancelButton on_cancel=on_cancel />
                </div>
            }
            .into_any(),
        }}
    }
}

#[component]
fn InboxCancelButton(on_cancel: Callback<()>) -> impl IntoView {
    let i18n = use_i18n();
    view! {
        <div class="flex justify-end">
            <Button
                variant=ButtonVariant::Ghost
                on_click=Callback::new(move |_| on_cancel.run(()))
                test_id="words-inbox-cancel-btn"
            >
                {t!(i18n, common.cancel)}
            </Button>
        </div>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use rstest::rstest;

    #[rstest]
    #[case::ocr_in_flight_wins(
        OcrState::Processing,
        AudioState::Idle,
        false,
        InboxViewStage::OcrProgress
    )]
    #[case::audio_in_flight(
        OcrState::Idle,
        AudioState::LoadingModel,
        false,
        InboxViewStage::AudioProgress
    )]
    #[case::error_after_audio(OcrState::Error, AudioState::Error, true, InboxViewStage::Failed)]
    #[case::waiting_generic(
        OcrState::Idle,
        AudioState::Idle,
        false,
        InboxViewStage::GenericProgress
    )]
    fn view_stage_priority_is_ocr_then_audio_then_error(
        #[case] ocr_state: OcrState,
        #[case] audio_state: AudioState,
        #[case] has_error: bool,
        #[case] expected: InboxViewStage,
    ) {
        assert_eq!(
            resolve_view_stage(ocr_state, audio_state, has_error),
            expected
        );
    }
}
