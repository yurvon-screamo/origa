use crate::i18n::*;
use crate::ui_components::{
    AudioPlayer, MarkdownText, MarkdownVariant, Text, TextSize, TranslatorText, TypographyVariant,
};
use leptos::prelude::*;
use origa::domain::QuizOption;
use std::collections::HashSet;

use super::card_type::CardType;
use super::lesson_card_header::LessonCardTagsQuiz;
use super::lesson_card_shell::LessonCardShell;
use super::next_card_button::NextCardButton;
use super::quiz_options::QuizOptions;
use super::quiz_result::QuizResult;
use super::quiz_result_display::QuizResultDisplay;

#[component]
pub fn PhraseCardView(
    card_type: CardType,
    audio_file: String,
    options: Vec<QuizOption>,
    show_result: Signal<bool>,
    selected_option: Option<usize>,
    on_select_option: Callback<usize>,
    on_dont_know: Callback<()>,
    dont_know_selected: bool,
    phrase_text: Option<String>,
    phrase_translation: Option<String>,
    #[prop(into)] known_kanji: Signal<HashSet<char>>,
    waiting_for_next: Signal<bool>,
    on_next_card: Callback<()>,
) -> impl IntoView {
    let i18n = use_i18n();
    // Bug A fix: pass the CDN path (not the resolved URL) to AudioPlayer so it
    // can prefetch into a `blob:` URL — see cdn_provider::resolve_audio_url.
    let audio_path = format!("phrases/audio/{}", audio_file);
    let options_stored = StoredValue::new(options);
    let phrase_text_stored = StoredValue::new(phrase_text);
    let phrase_translation_stored = StoredValue::new(phrase_translation);

    let quiz_result = move || {
        if dont_know_selected && show_result.get() {
            return QuizResult::DontKnow;
        }
        if let Some(selected) = selected_option {
            let opts = options_stored.get_value();
            if let Some(opt) = opts.get(selected) {
                return if opt.is_correct() {
                    QuizResult::Correct
                } else {
                    QuizResult::Incorrect
                };
            }
        }
        QuizResult::None
    };

    let tags_row: ChildrenFn = std::sync::Arc::new(move || {
        view! {
            <LessonCardTagsQuiz card_type=card_type />
        }
        .into_any()
    });

    view! {
        <LessonCardShell
            tags=tags_row
            test_id="lesson-card-root"
        >
        <div class="text-center mb-3 sm:mb-6">
            <AudioPlayer
                path=audio_path
                autoplay=true
                test_id=Signal::derive(|| "phrase-audio-player".to_string())
            />
            <Text size=TextSize::Default variant=TypographyVariant::Muted class="mt-4">
                {t!(i18n, lesson.choose_answer)}
            </Text>
        </div>

            <Show when=move || !show_result.get()>
                <QuizOptions
                    options=options_stored.get_value()
                    selected_option=selected_option
                    show_result=show_result
                    quiz_result=quiz_result()
                    on_select_option=on_select_option
                    on_dont_know=on_dont_know
                    dont_know_selected=Signal::derive(move || dont_know_selected)
                    known_kanji=known_kanji
                />
            </Show>

            <Show when=move || show_result.get()>
                <QuizResultDisplay quiz_result=quiz_result() />
            </Show>

            <Show when=move || show_result.get()>
                {move || match phrase_text_stored.get_value() {
                    Some(text) => {
                        let sentences = crate::utils::text_format::split_japanese_sentences(&text);
                        view! {
                            <div class="mt-4 p-3 bg-[var(--bg-secondary)] text-center flex flex-col gap-1">
                                {sentences.into_iter().map(|s| view! {
                                    <div><TranslatorText text=s /></div>
                                }).collect::<Vec<_>>()}
                            </div>
                        }.into_any()
                    },
                    None => view! { <div/> }.into_any(),
                }}
            </Show>

            <Show when=move || show_result.get()>
                {move || match phrase_translation_stored.get_value() {
                    Some(translation) => view! {
                        <div class="mt-2 p-3 bg-[var(--bg-secondary)] text-center">
                            <MarkdownText
                                content=Signal::derive(move || translation.clone())
                                variant=Signal::derive(|| MarkdownVariant::Default)
                                known_kanji=HashSet::new()
                                furigana=false
                            />
                        </div>
                    }.into_any(),
                    None => view! { <div/> }.into_any(),
                }}
            </Show>

            <Show when=move || waiting_for_next.get() && show_result.get()>
                <NextCardButton on_next_card=on_next_card />
            </Show>
        </LessonCardShell>
    }
}
