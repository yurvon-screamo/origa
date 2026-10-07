use super::super::shared::{
    CardListExtras, CardListViewConfig, ListGrouping, ListPage, card_list_view,
    create_card_list_context,
};
use super::vocabulary_card_item::VocabularyCardItem;
use crate::i18n::{td_string, use_i18n};
use crate::repository::HybridUserRepository;
use leptos::prelude::*;
use leptos_router::hooks::use_navigate;
use origa::domain::Card;

#[component]
pub fn WordsContent(refresh_trigger: RwSignal<u32>) -> impl IntoView {
    let i18n = use_i18n();
    let repository =
        use_context::<HybridUserRepository>().expect("repository context not provided");

    let ctx = create_card_list_context(
        repository,
        refresh_trigger,
        |card| matches!(card, Card::Vocabulary(_)),
        None,
    );

    // The router navigator must be captured in the component's reactive
    // scope — calling use_navigate() inside the click closure runs outside
    // any owner and silently kills the navigation after prevent_default
    // (the dead-link bug: tap/click on a word card did nothing).
    let navigate = StoredValue::new(use_navigate());

    let ctx_for_render = ctx.clone();
    let empty_message =
        Signal::derive(move || td_string!(i18n.get_locale(), words.words_not_found).to_string());

    let config = CardListViewConfig {
        test_id_prefix: "words",
        empty_message,
        grid_classes: Some(
            "grid grid-cols-1 md:grid-cols-2 lg:grid-cols-2 xl:grid-cols-3 2xl:grid-cols-4 gap-4 items-start",
        ),
    };
    card_list_view(
        ctx,
        ListPage::Words,
        ListGrouping::Flat,
        config,
        CardListExtras::default(),
        move |card| {
            let ctx = ctx_for_render.clone();
            let card_id = *card.card_id();
            view! {
                <VocabularyCardItem
                    study_card=card
                    on_open_detail_cb=Callback::new(move |w: String| {
                        let path = format!(
                            "/words/{}",
                            percent_encoding::utf8_percent_encode(
                                &w,
                                percent_encoding::NON_ALPHANUMERIC
                            )
                        );
                        navigate.get_value()(&path, Default::default());
                    })
                    native_language=ctx.native_lang
                    known_kanji=ctx.known_kanji.get()
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
