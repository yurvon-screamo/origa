use super::super::shared::{
    CardListExtras, CardListViewConfig, ListGrouping, ListPage, card_list_view,
    create_card_list_context, kanji_search_matcher,
};
use super::kanji_card_item::KanjiCardItem;
use crate::i18n::{t_string, use_i18n};
use crate::repository::HybridUserRepository;
use crate::store::AuthStore;
use leptos::prelude::*;
use origa::domain::{Card, CardType};

#[component]
pub fn KanjiContent(refresh_trigger: RwSignal<u32>) -> impl IntoView {
    let i18n = use_i18n();
    let repository =
        use_context::<HybridUserRepository>().expect("repository context not provided");

    let ctx = create_card_list_context(
        repository,
        refresh_trigger,
        |card| matches!(card, Card::Kanji(_)),
        None,
    );

    // On/kun readings resolve through the kanji dictionary; the readiness
    // signal re-runs a query typed while that dictionary still loads (#837).
    let kanji_ready = use_context::<AuthStore>()
        .expect("AuthStore not provided")
        .is_kanji_loaded;

    let ctx_for_render = ctx.clone();
    let empty_message = Signal::derive(move || t_string!(i18n, kanji_page.not_found).to_string());

    let config = CardListViewConfig {
        test_id_prefix: "kanji",
        empty_message,
        grid_classes: Some(
            "grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 2xl:grid-cols-5 gap-4 items-start",
        ),
    };
    card_list_view(
        ctx,
        ListPage::Kanji,
        ListGrouping::ByJlptLevel {
            card_type: CardType::Kanji,
        },
        config,
        CardListExtras {
            search_match: Some(kanji_search_matcher(kanji_ready)),
            ..CardListExtras::default()
        },
        move |card| {
            let ctx = ctx_for_render.clone();
            let card_id = *card.card_id();
            view! {
                <KanjiCardItem
                    study_card=card
                    native_language=ctx.native_lang
                    on_toggle_favorite=ctx.on_toggle_favorite
                    on_mark_as_known=Callback::new(move |_| ctx.on_mark_as_known.run(card_id))
                    on_delete=ctx.on_delete
                    is_deleting=ctx.is_deleting
                />
            }
            .into_any()
        },
    )
    .into_any()
}
