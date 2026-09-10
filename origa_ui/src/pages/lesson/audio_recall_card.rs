use crate::i18n::*;
use crate::ui_components::{
    Button, ButtonVariant, Card, MarkdownText, MarkdownVariant, Tag, TagVariant, Text, TextSize,
    TypographyVariant, speak_word, stop_current_audio, word_audio_available,
};
use leptos::prelude::*;
use leptos_icons::Icon;
use origa::domain::{Card as DomainCard, NativeLanguage};
use std::collections::HashSet;

use super::answer_display::{CardAnswerDisplay, extract_card_answer};
use super::card_type::CardType;
use super::lesson_state::LessonContext;
use super::next_card_button::NextCardButton;

/// Audio-recall card: the word is NEVER shown on the question side — the
/// learner hears it (autoplay + manual replay) and self-assesses with
/// «Не знаю» / «Знаю». The answer side is the classic one: the word and
/// its translation. Manual replay is NOT gated by the lesson mute: it is
/// an explicit user action, and a textless card without sound would be
/// unanswerable (see docs — AudioRecall mute matrix).
#[component]
pub fn AudioRecallCardView(
    card: DomainCard,
    show_result: Signal<bool>,
    on_answer: Callback<bool>,
    on_replay: Callback<()>,
    native_language: NativeLanguage,
    #[prop(into)] known_kanji: Signal<HashSet<char>>,
    #[prop(default = Signal::derive(|| false))] waiting_for_next: Signal<bool>,
    #[prop(default = Callback::new(|_: ()| {}))] on_next_card: Callback<()>,
) -> impl IntoView {
    let i18n = use_i18n();
    let card_type = CardType::from(&card);
    let lang = native_language;

    let question_text = match card.question(&lang) {
        Ok(q) => q.text().to_string(),
        Err(_) => String::new(),
    };
    let is_na_adj = super::na_adjective_helper::is_na_adjective_card(&card);
    let display_word = if is_na_adj {
        super::na_adjective_helper::append_na_suffix(&question_text)
    } else {
        question_text.clone()
    };
    let word_stored = StoredValue::new(display_word);

    let answer_data = extract_card_answer(&card, &lang, &card_type);
    let answer_translations = StoredValue::new(answer_data.translations);
    let answer_description = StoredValue::new(answer_data.description);
    let answer_text = StoredValue::new(answer_data.text);

    let lesson_ctx = use_context::<LessonContext>();
    // Autoplay once when the question side mounts. Guards mirror the
    // availability predicate (pitch audio OR TTS) plus the lesson mute —
    // a muted user never reaches this component in the first place (mode
    // sampling), but the guard keeps the invariant if state drifts.
    Effect::new(move |_| {
        let is_muted = lesson_ctx
            .as_ref()
            .map(|ctx| ctx.is_muted.get_untracked())
            .unwrap_or(false);
        let answered = show_result.get();
        if !answered && !is_muted && word_audio_available(&question_text) {
            speak_word(&question_text, 1.0);
        }
    });

    on_cleanup(move || {
        stop_current_audio();
    });

    view! {
        <div class="flex flex-col">
            <div class="flex items-center gap-2 flex-wrap min-w-0 mb-2 px-1">
                <Tag variant=Signal::derive(move || card_type.tag_variant())>
                    {card_type.label(&i18n)}
                </Tag>
                <Tag variant=Signal::derive(move || TagVariant::Filled)>
                    {t!(i18n, lesson.audio_tag)}
                </Tag>
            </div>
            <Card class=Signal::derive(|| super::LESSON_CARD_CLASS.to_string()) shadow=true test_id="audio-recall-card-root">

            <div class="flex-1 flex flex-col justify-center">
                <div class="text-center mb-3 sm:mb-6">
                    <button
                        data-testid="audio-recall-play-btn"
                        class="audio-player-btn p-3 sm:p-4 rounded-full border transition-all cursor-pointer hover:bg-[var(--bg-hover)]"
                        on:click=move |_| on_replay.run(())
                    >
                        <Icon icon=icondata::LuVolume2 width="1.5em" height="1.5em" />
                    </button>
                    <Show when=move || !show_result.get()>
                        <Text size=TextSize::Default variant=TypographyVariant::Muted class="mt-4">
                            {t!(i18n, lesson.listen_word)}
                        </Text>
                        <Text size=TextSize::Small variant=TypographyVariant::Muted class="mt-1">
                            {t!(i18n, lesson.space_key)}
                        </Text>
                    </Show>
                </div>

                <Show when=move || !show_result.get()>
                    <div class="grid grid-cols-2 gap-3">
                        <Button
                            test_id=Signal::derive(|| "audio-recall-dont-know-btn".to_string())
                            variant=Signal::derive(|| ButtonVariant::Default)
                            disabled=Signal::derive(move || show_result.get())
                            on_click=Callback::new(move |_| on_answer.run(false))
                        >
                            {t!(i18n, lesson.dont_know_rating)} <span class="kbd-hint">"[1]"</span>
                        </Button>

                        <Button
                            test_id=Signal::derive(|| "audio-recall-know-btn".to_string())
                            variant=Signal::derive(|| ButtonVariant::Olive)
                            disabled=Signal::derive(move || show_result.get())
                            on_click=Callback::new(move |_| on_answer.run(true))
                        >
                            {t!(i18n, lesson.know)} <span class="kbd-hint">"[2]"</span>
                        </Button>
                    </div>
                </Show>

                <Show when=move || show_result.get()>
                    // The word itself is part of the ANSWER side — the
                    // learner checks what they heard against the written form.
                    <div class="text-center mb-2">
                        <MarkdownText
                            content=Signal::derive(move || word_stored.get_value())
                            known_kanji=known_kanji.get()
                            variant=Signal::derive(|| MarkdownVariant::Large)
                        />
                    </div>

                    <CardAnswerDisplay
                        translations=Signal::derive(move || answer_translations.get_value())
                        description=Signal::derive(move || answer_description.get_value())
                        text=Signal::derive(move || answer_text.get_value())
                        known_kanji=known_kanji
                    />
                </Show>

                <Show when=move || waiting_for_next.get() && show_result.get()>
                    <NextCardButton on_next_card=on_next_card />
                </Show>
            </div>
        </Card>
        </div>
    }
}
