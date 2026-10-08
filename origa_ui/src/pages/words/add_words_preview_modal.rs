use crate::core::share_intake::{self, ShareWire};
use crate::i18n::{t, use_i18n};
use crate::pages::words::add_words_preview_modal_handlers::create_preview_modal_handlers;
use crate::pages::words::add_words_preview_modal_state::{
    AnalysisStage, InputMode, PreviewModalState, analysis_stage,
};
use crate::pages::words::analyzed_word_item::AnalyzedWordItem;
use crate::pages::words::anki_import_stage::AnkiImportStage;
use crate::pages::words::audio_input_stage::AudioInputStage;
use crate::pages::words::audio_transcribe::{AudioState, cancel_whisper_loading};
use crate::pages::words::image_input_stage::ImageInputStage;
use crate::pages::words::inbox::{
    InboxSeamGuard, InboxSignals, InboxStageView, register_inbox_seam,
};
use crate::pages::words::ocr_processing::OcrState;
use crate::pages::words::transcript::{
    TranscriptDecision, join_selected, sentence_has_unknown_content, split_sentences,
    transcript_entry,
};
use crate::pages::words::transcript_view::TranscriptStageView;
use crate::repository::HybridUserRepository;
use crate::ui_components::{
    Alert, AlertType, Button, ButtonVariant, Drawer, Input, TabItem, Tabs, Text, TextSize,
    ToastContainer, ToastData, ToastType, TypographyVariant,
};
use leptos::ev::MouseEvent;
use leptos::prelude::*;
use leptos::task::spawn_local;
use origa::domain::User;
use origa::traits::UserRepository;
use origa::use_cases::AnalyzedWord;
use std::collections::HashSet;
use std::sync::atomic::Ordering;

#[component]
pub fn AddWordsPreviewModal(
    is_open: RwSignal<bool>,
    refresh_trigger: RwSignal<u32>,
) -> impl IntoView {
    let i18n = use_i18n();
    let repository =
        use_context::<HybridUserRepository>().expect("repository context not provided");

    let current_user: RwSignal<Option<User>> = RwSignal::new(None);
    let repo_for_effect = repository.clone();
    let disposed = StoredValue::new(());

    Effect::new(move |_| {
        let repo = repo_for_effect.clone();
        spawn_local(async move {
            if let Ok(Some(user)) = repo.get_current_user().await {
                if disposed.is_disposed() {
                    return;
                }
                current_user.set(Some(user));
            }
        });
    });

    let known_kanji = Memo::new(move |_| {
        current_user
            .get()
            .map(|u| u.knowledge_set().get_known_kanji())
            .unwrap_or_default()
    });

    let state = PreviewModalState::new(is_open, refresh_trigger);
    let analyzed_words = state.analyzed_words;
    let input_text = state.input_text;
    let is_analyzing = state.is_analyzing;
    let error_message = state.error_message;
    let selected_words = state.selected_words;
    let is_creating = state.is_creating;
    let input_mode = state.input_mode;
    let active_tab = state.active_tab;
    let handlers = create_preview_modal_handlers(state.clone(), is_open);

    let inbox = InboxSignals::new();
    let toasts: RwSignal<Vec<ToastData>> = RwSignal::new(Vec::new());

    // Transcript stage (AU-2): long audio transcriptions land here first so
    // the user picks which sentences to analyze instead of facing the word
    // preview of an hour-long text.
    let transcript_sentences: RwSignal<Option<Vec<String>>> = RwSignal::new(None);
    let transcript_selected: RwSignal<HashSet<usize>> = RwSignal::new(HashSet::new());

    Effect::new({
        let state = state.clone();
        move |_| {
            if !is_open.get() {
                state.reset();
                inbox.reset();
                transcript_sentences.set(None);
                transcript_selected.set(HashSet::new());
            }
        }
    });

    // Transcript stage actions (AU-2). Hoisted before the view: Callback is
    // Copy, but the inline `state.clone()` moved the non-Copy state into
    // the view closure and turned it FnOnce.
    let on_analyze_selected_sentences = Callback::new({
        let state = state.clone();
        move |_: ()| {
            let sentences = transcript_sentences.get().unwrap_or_default();
            let text = join_selected(&sentences, &transcript_selected.get());
            transcript_sentences.set(None);
            transcript_selected.set(HashSet::new());
            state.set_extracted_text(text);
        }
    });
    let on_use_all_text = Callback::new({
        let state = state.clone();
        move |_: ()| {
            let sentences = transcript_sentences.get().unwrap_or_default();
            let text = sentences.join("");
            transcript_sentences.set(None);
            transcript_selected.set(HashSet::new());
            state.set_extracted_text(text);
        }
    });

    // Audio transcriptions land on the transcript screen; image OCR goes
    // straight to analysis (a page photo has no sentence-selection value).
    let on_audio_text_extracted = {
        let state = state.clone();
        Callback::new(move |text: String| {
            let sentences = split_sentences(&text);
            match transcript_entry(sentences.len()) {
                TranscriptDecision::DirectAnalysis => {
                    transcript_sentences.set(None);
                    state.set_extracted_text(text);
                },
                TranscriptDecision::ShowScreen => {
                    let known = known_kanji.get();
                    let preselected: HashSet<usize> = sentences
                        .iter()
                        .enumerate()
                        .filter(|(_, sentence)| sentence_has_unknown_content(sentence, &known))
                        .map(|(index, _)| index)
                        .collect();
                    transcript_sentences.set(Some(sentences));
                    transcript_selected.set(preselected);
                },
            }
        })
    };

    // External share intake (IN-2/IN-3): consume parked payloads as they
    // arrive (mount + warm event delivery while the page is open). The
    // parking slot is non-reactive (thread_local), so a poll interval is
    // the consumption trigger.
    static SHARE_ERROR_TOAST_SEQ: std::sync::atomic::AtomicUsize =
        std::sync::atomic::AtomicUsize::new(0);
    {
        let state = state.clone();
        let disposed = state.disposed;
        let on_audio_text = on_audio_text_extracted;
        let i18n_for_share = i18n;
        leptos::task::spawn_local(async move {
            loop {
                gloo_timers::future::TimeoutFuture::new(300).await;
                // Exit when this modal instance unmounts — otherwise the
                // loop outlives the page and steals shares from the next
                // instance's signals (dead instance, no subscribers).
                if disposed.is_disposed() {
                    return;
                }
                let Some(payload) = share_intake::take_share() else {
                    continue;
                };
                let state = state.clone();
                let on_audio_text = on_audio_text;
                let i18n = i18n_for_share;
                let toasts = toasts;
                let is_open = is_open;
                let inbox = inbox;
                spawn_local(async move {
                    match payload {
                        ShareWire::Text { text } => {
                            let route = crate::pages::words::inbox::wire_route_text(&text);
                            crate::pages::words::inbox::seam::execute_route(
                                route,
                                i18n,
                                &state,
                                is_open,
                                &inbox,
                                on_audio_text,
                            );
                        },
                        ShareWire::File {
                            file_name,
                            mime,
                            cache_path,
                        } => match share_intake::read_shared_bytes(&cache_path).await {
                            Ok(bytes) => {
                                let kind = crate::pages::words::inbox::wire_route_file(
                                    &file_name, &mime, bytes,
                                );
                                if let Some(kind) = kind {
                                    let route = crate::pages::words::inbox::route_payload(
                                        crate::pages::words::inbox::InboxPayload { kind },
                                    );
                                    crate::pages::words::inbox::seam::execute_route(
                                        route,
                                        i18n,
                                        &state,
                                        is_open,
                                        &inbox,
                                        on_audio_text,
                                    );
                                }
                            },
                            Err(e) => {
                                tracing::warn!(error = %e, "share-intake: file read failed")
                            },
                        },
                        ShareWire::Error { message } => {
                            tracing::warn!(message = %message, "share-intake: host reported error");
                            toasts.update(|list| {
                                list.push(ToastData {
                                    id: SHARE_ERROR_TOAST_SEQ.fetch_add(1, Ordering::Relaxed),
                                    toast_type: ToastType::Info,
                                    title: i18n
                                        .get_keys_untracked()
                                        .common()
                                        .error()
                                        .inner()
                                        .to_string(),
                                    message,
                                    duration_ms: Some(6000),
                                    closable: true,
                                });
                            });
                        },
                        ShareWire::None => {},
                    }
                });
            }
        });
    }

    // The e2e seam registers once at mount and unregisters via its guard's
    // Drop when the drawer component is disposed.
    let seam_guard = StoredValue::new_local(None::<InboxSeamGuard>);
    Effect::new({
        let state = state.clone();
        move |_| {
            seam_guard.set_value(register_inbox_seam(
                state.clone(),
                is_open,
                inbox,
                toasts,
                i18n,
                on_audio_text_extracted,
            ));
        }
    });

    let has_analyzed = state.has_analyzed;

    let tabs = Signal::derive(move || {
        let mut items = vec![
            TabItem {
                id: "text".to_string(),
                label: i18n.get_keys().words().tab_text().inner().to_string(),
            },
            TabItem {
                id: "image".to_string(),
                label: i18n.get_keys().words().tab_image().inner().to_string(),
            },
        ];
        items.push(TabItem {
            id: "anki".to_string(),
            label: i18n.get_keys().words().tab_anki().inner().to_string(),
        });
        items.push(TabItem {
            id: "audio".to_string(),
            label: i18n.get_keys().words().tab_audio().inner().to_string(),
        });
        items
    });

    Effect::new({
        let state = state.clone();
        move || {
            let tab = state.active_tab.get();
            state.input_mode.set(if tab == "image" {
                InputMode::Image
            } else if tab == "anki" {
                InputMode::Anki
            } else if tab == "audio" {
                InputMode::Audio
            } else {
                InputMode::Text
            });
        }
    });

    let on_text_extracted = {
        let state = state.clone();
        Callback::new(move |text: String| {
            state.set_extracted_text(text);
        })
    };

    let on_ocr_error = {
        Callback::new(move |_msg: String| {
            error_message.set(None);
        })
    };

    let on_switch_to_text = {
        Callback::new(move |_| {
            input_mode.set(InputMode::Text);
            active_tab.set("text".to_string());
        })
    };

    // Inbox fallbacks: cancel aborts the zero-tap run and returns to the
    // source tabs; open-manually does the same after a failure. Cancel must
    // (a) invalidate the in-flight run — a late OCR/STT result would
    // otherwise feed the next payload — and (b) clear the OCR state, whose
    // cancelled pipeline never reaches its own completion branch and would
    // otherwise report Processing forever, rejecting every next payload.
    // reset() goes first: it clears cancel_requested, and the flag is then
    // re-raised so an orphaned OCR run aborted mid-model-download still
    // stops at its next cancellation checkpoint.
    let on_inbox_cancel = {
        Callback::new(move |_: ()| {
            inbox.generation.update(|g| *g = g.wrapping_add(1));
            inbox.ocr_loading_state.reset();
            inbox.ocr_loading_state.cancel_requested.set(true);
            inbox.ocr_state.set(OcrState::Idle);
            inbox.audio_state.set(AudioState::Idle);
            cancel_whisper_loading();
            inbox.active.set(false);
            inbox.error.set(None);
        })
    };

    let on_inbox_open_manually = {
        Callback::new(move |_: ()| {
            inbox.active.set(false);
            inbox.error.set(None);
        })
    };

    view! {
        <Drawer
            is_open=is_open
            title=Signal::derive(move || i18n.get_keys().words().add_words().inner().to_string())
            test_id="words-add-drawer"
        >
            <div class="space-y-4">
                {move || {
                    let words = analyzed_words.get();
                    let stage = analysis_stage(words.len(), is_analyzing.get());
                    match stage {
                        AnalysisStage::Analyzing => view! {
                            <div class="space-y-4">
                                <Show when=move || !inbox.active.get()>
                                    <Tabs tabs=tabs active=active_tab test_id=Signal::derive(|| "words-add-tabs".to_string()) class="tabs--scrollable".to_string() />
                                </Show>
                                <div class="flex items-center justify-center py-8">
                                    <Text size=TextSize::Default variant=TypographyVariant::Muted>
                                        {t!(i18n, words.analyzing)}
                                    </Text>
                                </div>
                            </div>
                        }.into_any(),
                        AnalysisStage::Preview => view! {
                            <PreviewStage
                                analyzed_words=words
                                selected_words=selected_words
                                known_kanji=known_kanji.get()
                                is_creating=is_creating
                                on_word_toggle=handlers.on_word_toggle
                                on_cancel=handlers.on_cancel
                                on_create=handlers.on_create
                            />
                        }.into_any(),
                        AnalysisStage::Input => view! {
                            <div class="space-y-4">
                                {move || {
                                    let no_words_after_analysis = has_analyzed.get() && analyzed_words.get().is_empty();
                                    if no_words_after_analysis || inbox.empty_text.get() {
                                        Some(view! {
                                            <Alert
                                                alert_type=Signal::derive(|| AlertType::Warning)
                                                title=Signal::derive(move || i18n.get_keys().words().words_not_found().inner().to_string())
                                                message=Signal::derive(move || i18n.get_keys().words().words_not_found_hint().inner().to_string())
                                                test_id=Signal::derive(|| "words-no-results".to_string())
                                            />
                                        })
                                    } else {
                                        None
                                    }
                                }}
                                {move || {
                                    if transcript_sentences.get().is_some() {
                                        view! {
                                            <TranscriptStageView
                                                sentences=Signal::derive(move || {
                                                    transcript_sentences
                                                        .get()
                                                        .unwrap_or_default()
                                                })
                                                selected=transcript_selected
                                                on_analyze_selected=on_analyze_selected_sentences
                                                on_use_all=on_use_all_text
                                                on_back=Callback::new(move |_: ()| {
                                                    // Exit the whole zero-tap
                                                    // run: otherwise the inbox
                                                    // view shows an eternal
                                                    // "processing" spinner.
                                                    inbox.active.set(false);
                                                    transcript_sentences.set(None);
                                                    transcript_selected.set(HashSet::new());
                                                })
                                            />
                                        }.into_any()
                                    } else if inbox.active.get() {
                                        view! {
                                            <InboxStageView
                                                inbox=inbox
                                                on_cancel=on_inbox_cancel
                                                on_open_manually=on_inbox_open_manually
                                            />
                                        }.into_any()
                                    } else {
                                        view! {
                                            <Tabs tabs=tabs active=active_tab test_id=Signal::derive(|| "words-add-tabs".to_string()) class="tabs--scrollable".to_string() />
                                            {move || {
                                                let mode = input_mode.get();
                                                match mode {
                                                    InputMode::Text => view! {
                                                        <InputStage
                                                            input_text=input_text
                                                            is_analyzing=is_analyzing
                                                            error_message=error_message
                                                            on_analyze=handlers.on_analyze
                                                        />
                                                    }.into_any(),
                                                    InputMode::Anki => {
                                                        view! {
                                                            <AnkiImportStage
                                                                is_open=is_open
                                                                refresh_trigger=refresh_trigger
                                                                test_id=Signal::derive(|| "words-drawer-anki".to_string())
                                                            />
                                                        }.into_any()
                                                    },
                                                    InputMode::Image => view! {
                                                        <ImageInputStage
                                                            is_open=is_open
                                                            on_text_extracted=on_text_extracted
                                                            on_error=on_ocr_error
                                                            on_switch_to_text=on_switch_to_text
                                                        />
                                                    }.into_any(),
                                                    InputMode::Audio => view! {
                                                        <AudioInputStage
                                                            is_open=Signal::derive(move || is_open.get())
                                                            on_text_extracted=on_audio_text_extracted
                                                            on_error=on_ocr_error
                                                            on_switch_to_text=on_switch_to_text
                                                        />
                                                    }.into_any(),
                                                }
                                            }}
                                        }.into_any()
                                    }
                                }}
                            </div>
                        }.into_any(),
                    }
                }}
            </div>
            <ToastContainer toasts=toasts duration_ms=4000 />
        </Drawer>
    }
}

#[component]
fn PreviewStage(
    analyzed_words: Vec<AnalyzedWord>,
    selected_words: RwSignal<std::collections::HashSet<String>>,
    known_kanji: std::collections::HashSet<char>,
    is_creating: RwSignal<bool>,
    on_word_toggle: Callback<String>,
    on_cancel: Callback<MouseEvent>,
    on_create: Callback<()>,
) -> impl IntoView {
    let i18n = use_i18n();
    let analyzed_words_count = analyzed_words.len();
    let new_words_count = analyzed_words.iter().filter(|w| !w.is_known).count();

    view! {
        <div>
            <Text size=TextSize::Small variant=TypographyVariant::Muted>
                {move || {
                    i18n.get_keys().words().found_words().inner().to_string()
                        .replacen("{}", &analyzed_words_count.to_string(), 1)
                        .replacen("{}", &new_words_count.to_string(), 1)
                }}
            </Text>
        </div>
        <div class="space-y-2">
            <For
                each=move || analyzed_words.clone()
                key=|word| word.base_form.clone()
                children=move |word| {
                    let base_form = word.base_form.clone();
                    view! {
                        <AnalyzedWordItem
                            analyzed_word=word
                            selected_words=selected_words
                            known_kanji=known_kanji.clone()
                            on_toggle=Callback::new(move |_| on_word_toggle.run(base_form.clone()))
                        />
                    }
                }
            />
        </div>
        <div class="flex gap-2 justify-between">
            <Button
                variant=ButtonVariant::Ghost
                on_click=on_cancel
                test_id="words-drawer-cancel-btn"
            >
                {t!(i18n, words.cancel)}
            </Button>
            <Button
                variant=ButtonVariant::Olive
                disabled=Signal::derive(move || {
                    selected_words.get().is_empty()
                        || is_creating.get()
                })
                on_click=Callback::new(move |_| on_create.run(()))
                test_id="words-drawer-add-btn"
            >
                {move || {
                    if is_creating.get() {
                        t!(i18n, words.creating).into_any()
                    } else {
                        t!(i18n, words.add_selected).into_any()
                    }
                }}
            </Button>
        </div>
    }
}

#[component]
fn InputStage(
    input_text: RwSignal<String>,
    is_analyzing: RwSignal<bool>,
    error_message: RwSignal<Option<String>>,
    on_analyze: Callback<()>,
) -> impl IntoView {
    let i18n = use_i18n();
    view! {
        <div>
            <Text size=TextSize::Small variant=TypographyVariant::Muted class=Signal::derive(|| "mb-2".to_string())>
                {t!(i18n, words.enter_japanese)}
            </Text>
            <Input
                value=input_text
                placeholder=Signal::derive(|| "例えば、本を読みます。".to_string())
                rows=Signal::derive(|| Some(10))
                test_id="words-drawer-textarea"
            />
        </div>
        {move || {
            error_message.get().map(move |msg| view! {
                <Alert
                    alert_type=Signal::derive(|| AlertType::Error)
                    title=Signal::derive(move || i18n.get_keys().words().error().inner().to_string())
                    message=Signal::derive(move || msg.clone())
                />
            })
        }}
        <Button
            variant=ButtonVariant::Olive
            disabled=Signal::derive(move || {
                input_text.get().trim().is_empty()
                    || is_analyzing.get()
            })
            on_click=Callback::new(move |_| on_analyze.run(()))
            test_id="words-drawer-analyze-btn"
        >
            {move || {
                if is_analyzing.get() {
                    t!(i18n, words.analyzing).into_any()
                } else {
                    t!(i18n, words.analyze).into_any()
                }
            }}
        </Button>
    }
}
