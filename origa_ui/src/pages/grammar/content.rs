use super::super::shared::{card_list_view, create_card_list_context};
use super::grammar_card_item::GrammarCardItem;
use super::grammar_detail_drawer::GrammarDetailDrawer;
use crate::i18n::{td_string, use_i18n};
use crate::repository::HybridUserRepository;
use leptos::prelude::*;
use origa::domain::Card;

#[component]
pub fn GrammarContent(refresh_trigger: RwSignal<u32>) -> impl IntoView {
    let i18n = use_i18n();
    let repository =
        use_context::<HybridUserRepository>().expect("repository context not provided");

    let ctx = create_card_list_context(
        repository,
        refresh_trigger,
        |card| matches!(card, Card::Grammar(_)),
        None,
    );

    let selected_card: RwSignal<Option<origa::domain::StudyCard>> = RwSignal::new(None);

    let all_cards = ctx.all_cards;
    let selected_card_id: Memo<Option<ulid::Ulid>> =
        Memo::new(move |_| selected_card.get().map(|c| *c.card_id()));
    Effect::new(move |_| {
        if let Some(id) = selected_card_id.get() {
            let cards = all_cards.get();
            if let Some(updated) = cards.iter().find(|c| *c.card_id() == id) {
                selected_card.set(Some(updated.clone()));
            }
        }
    });

    let ctx_for_render = ctx.clone();
    let empty_message =
        Signal::derive(move || td_string!(i18n.get_locale(), grammar_page.not_found).to_string());

    let on_close_detail = Callback::new(move |_: ()| selected_card.set(None));

    let current_user = ctx.current_user;
    let native_lang = ctx.native_lang;
    let known_kanji = ctx.known_kanji;
    let on_toggle_favorite = ctx.on_toggle_favorite;
    let on_mark_as_known = ctx.on_mark_as_known;
    let on_delete = ctx.on_delete;
    let is_deleting = ctx.is_deleting;

    let main_view = card_list_view(ctx, true, "grammar", empty_message, move |card| {
        let ctx = ctx_for_render.clone();
        let card_id = *card.card_id();
        let card_for_detail = card.clone();
        view! {
                <GrammarCardItem
                    study_card=card
                    native_language=ctx.native_lang
                    known_kanji=ctx.known_kanji.get()
                    on_toggle_favorite=ctx.on_toggle_favorite
                    on_mark_as_known=Callback::new(move |_| ctx.on_mark_as_known.run(card_id))
                    on_delete=ctx.on_delete
                    is_deleting=ctx.is_deleting
                    on_open_detail=Callback::new(move |_| selected_card.set(Some(card_for_detail.clone())))
                />
            }
            .into_any()
    });

    view! {
        {main_view}

        <Show when=move || selected_card.get().is_some()>
            {move || {
                let card = selected_card.get()?;
                let card_id = *card.card_id();
                Some(view! {
                    <GrammarDetailDrawer
                        study_card=card
                        native_language=native_lang
                        known_kanji=known_kanji.get()
                        user=current_user.get()
                        on_toggle_favorite=on_toggle_favorite
                        on_mark_as_known=Callback::new(move |_| on_mark_as_known.run(card_id))
                        on_delete=on_delete
                        is_deleting=is_deleting
                        on_close=on_close_detail
                    />
                }.into_any())
            }}
        </Show>
    }
    .into_any()
}
