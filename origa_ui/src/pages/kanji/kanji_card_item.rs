use super::super::shared::{CardStatus, DeleteRequest};
use crate::i18n::use_i18n;
use crate::ui_components::{
    Card, CardActionBar, DeleteConfirmModal, Text, TextSize, TypographyVariant,
};
use leptos::prelude::*;
use origa::domain::{Card as DomainCard, NativeLanguage, StudyCard};
use ulid::Ulid;

const DESCRIPTION_PREVIEW_LEN: usize = 100;
const RADICALS_MAX_LEN: usize = 20;

#[component]
pub fn KanjiCardItem(
    study_card: StudyCard,
    #[prop(into)] native_language: Signal<NativeLanguage>,
    on_toggle_favorite: Callback<Ulid>,
    on_mark_as_known: Callback<()>,
    on_delete: Callback<DeleteRequest>,
    is_deleting: Signal<bool>,
    #[prop(into)] on_open_detail: Callback<()>,
) -> impl IntoView {
    let i18n = use_i18n();
    let card_id = *study_card.card_id();
    let is_favorite = study_card.is_favorite();

    let is_delete_modal_open = RwSignal::new(false);
    let is_history_open = RwSignal::new(false);

    let confirm_delete = Callback::new(move |_| {
        on_delete.run(DeleteRequest {
            card_id,
            on_success: Callback::new(move |_| is_delete_modal_open.set(false)),
        })
    });

    let (kanji_char, radicals) = match study_card.card() {
        DomainCard::Kanji(kanji_card) => (
            kanji_card.kanji().text().to_string(),
            kanji_card.radicals_chars().into_iter().collect::<String>(),
        ),
        _ => ("?".to_string(), String::new()),
    };

    let status = CardStatus::from_study_card(&study_card);

    let study_card_for_desc = study_card.clone();
    let description_preview = Memo::new(move |_| {
        let lang = native_language.get();
        match study_card_for_desc.card() {
            DomainCard::Kanji(kanji_card) => kanji_card
                .description(&lang)
                .ok()
                .map(|d| d.text().to_string())
                .unwrap_or_default(),
            _ => String::new(),
        }
    });

    let show_radicals = !radicals.is_empty() && radicals.len() <= RADICALS_MAX_LEN;
    let radicals_text = Signal::derive({
        let radicals = radicals.clone();
        move || {
            i18n.get_keys()
                .shared()
                .radicals_label()
                .inner()
                .to_string()
                .replacen("{}", &radicals, 1)
        }
    });

    view! {
        <Card class="p-4 cursor-pointer" test_id="kanji-card-item" on:click=move |_: leptos::ev::MouseEvent| on_open_detail.run(())>
            <div class="flex items-start gap-3">
                <div class="w-12 h-12 flex items-center justify-center border border-[var(--border-dark)] bg-[var(--bg-paper)] shrink-0">
                    <span class="text-2xl font-serif">{kanji_char}</span>
                </div>
                <div class="min-w-0 flex-1">
                    <Text
                        size=TextSize::Small
                        variant=TypographyVariant::Muted
                        class=Signal::derive(|| "line-clamp-2".to_string())
                    >
                        {move || {
                            let desc = description_preview.get();
                            if desc.len() > DESCRIPTION_PREVIEW_LEN {
                                let truncated: String = desc.chars().take(80).collect();
                                format!("{}…", truncated)
                            } else {
                                desc
                            }
                        }}
                    </Text>
                    <Show when=move || show_radicals>
                        <Text size=TextSize::Small variant=TypographyVariant::Muted class=Signal::derive(|| "mt-0.5".to_string())>
                            {radicals_text}
                        </Text>
                    </Show>
                </div>
            </div>
            <div class="mt-2">
                <CardActionBar
                    tag_variant=Signal::derive(move || status.tag_variant())
                    tag_label=Signal::derive(move || status.label(&i18n))
                    is_favorite=Signal::derive(move || is_favorite)
                    on_toggle_favorite=Callback::new(move |_| on_toggle_favorite.run(card_id))
                    show_mark_as_known=Signal::derive(move || status != CardStatus::Learned)
                    on_mark_as_known=Callback::new(move |_| on_mark_as_known.run(()))
                    on_history=Callback::new(move |_| is_history_open.set(true))
                    on_delete=Callback::new(move |_| is_delete_modal_open.set(true))
                    test_id=Signal::derive(|| "kanji-card-item".to_string())
                />
            </div>
        </Card>
        <DeleteConfirmModal
            test_id="kanji-delete-modal"
            is_open=is_delete_modal_open
            is_deleting=is_deleting
            on_confirm=confirm_delete
            on_close=Callback::new(move |_| is_delete_modal_open.set(false))
        />
    }
}
