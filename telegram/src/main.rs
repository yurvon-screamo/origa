mod bot;
mod dialogue;
mod handlers;
mod service;

use dialogue::{DialogueState, LessonMode};
use handlers::{
    Command,
    add_from_text::add_from_text_handler,
    callback_handler,
    grammar::grammar_list_handler,
    handle_duolingo_token, help_handler,
    kanji::handle_kanji_list,
    lesson::{start_fixation, start_lesson},
    main_menu_handler, profile_handler, start_handler,
    vocabulary::vocabulary_list_handler,
};
use service::OrigaServiceProvider;
use teloxide::dispatching::dialogue::{InMemStorage, enter};
use teloxide::prelude::*;
use tracing::info;

use crate::handlers::vocabulary::callback::handle_vocabulary_search;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt().init();

    let bot = Bot::new(std::env::var("ORIGA_TELOXIDE_TOKEN").unwrap());
    info!("Starting Origa Telegram bot...");
    tokio::fs::create_dir_all("./data").await?;

    let message_handler = Update::filter_message()
        .branch(
            dptree::entry()
                .filter_command::<Command>()
                .branch(dptree::case![Command::Help].endpoint(help_handler))
                .branch(dptree::case![Command::Start].endpoint(start_handler)),
        )
        .branch(dptree::case![DialogueState::Idle].endpoint(main_menu_handler))
        .branch(
            dptree::case![DialogueState::VocabularyList {
                page,
                items_per_page,
                filter
            }]
            .endpoint(vocabulary_endpoint),
        )
        .branch(
            dptree::case![DialogueState::KanjiList {
                level,
                page,
                items_per_page
            }]
            .endpoint(kanji_endpoint),
        )
        .branch(
            dptree::case![DialogueState::AddFromText { pending_words }]
                .endpoint(add_from_text_endpoint),
        )
        .branch(
            dptree::case![DialogueState::GrammarList {
                page,
                items_per_page
            }]
            .endpoint(grammar_endpoint),
        )
        .branch(
            dptree::case![DialogueState::VocabularySearch {
                page,
                items_per_page,
                query
            }]
            .endpoint(vocabulary_search_endpoint),
        )
        .branch(
            dptree::case![DialogueState::Lesson {
                mode,
                card_ids,
                current_index,
                showing_answer,
                new_count,
                review_count
            }]
            .endpoint(lesson_endpoint),
        )
        .branch(dptree::case![DialogueState::Profile { current_view }].endpoint(profile_endpoint))
        .branch(dptree::case![DialogueState::DuolingoConnect].endpoint(duolingo_connect_endpoint));

    let callback_query_handler = Update::filter_callback_query().endpoint(callback_handler);

    let handler = enter::<Update, InMemStorage<DialogueState>, DialogueState, _>()
        .branch(message_handler)
        .branch(callback_query_handler);

    Dispatcher::builder(bot.clone(), handler)
        .dependencies(dptree::deps![InMemStorage::<DialogueState>::new()])
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;

    Ok(())
}

async fn vocabulary_endpoint(
    bot: Bot,
    msg: Message,
    dialogue: handlers::OrigaDialogue,
    (page, items_per_page, filter): (usize, usize, String),
) -> ResponseResult<()> {
    let telegram_id = msg.chat.id.0 as u64;
    let username = msg
        .from
        .as_ref()
        .map(|u| u.first_name.as_str())
        .unwrap_or("User");

    let provider = OrigaServiceProvider::instance().await;
    let session = provider
        .get_or_create_session(telegram_id, username)
        .await?;

    vocabulary_list_handler(bot, msg, dialogue, (page, items_per_page, filter), session).await
}

async fn add_from_text_endpoint(
    bot: Bot,
    msg: Message,
    dialogue: handlers::OrigaDialogue,
    pending_words: Vec<String>,
) -> ResponseResult<()> {
    add_from_text_handler(bot, msg, dialogue, pending_words).await
}

async fn kanji_endpoint(
    bot: Bot,
    msg: Message,
    (_level, page, items_per_page): (Option<origa::domain::JapaneseLevel>, usize, usize),
) -> ResponseResult<()> {
    let telegram_id = msg.chat.id.0 as u64;
    let username = msg
        .from
        .as_ref()
        .map(|u| u.first_name.as_str())
        .unwrap_or("User");

    let provider = OrigaServiceProvider::instance().await;
    let session = provider
        .get_or_create_session(telegram_id, username)
        .await?;

    handle_kanji_list(bot, msg, (page, items_per_page), session).await
}

async fn grammar_endpoint(
    bot: Bot,
    msg: Message,
    dialogue: handlers::OrigaDialogue,
    (page, items_per_page): (usize, usize),
) -> ResponseResult<()> {
    let telegram_id = msg.chat.id.0 as u64;
    let username = msg
        .from
        .as_ref()
        .map(|u| u.first_name.as_str())
        .unwrap_or("User");

    let provider = OrigaServiceProvider::instance().await;
    let session = provider
        .get_or_create_session(telegram_id, username)
        .await?;

    grammar_list_handler(bot, msg, dialogue, (page, items_per_page), session).await
}

async fn lesson_endpoint(
    bot: Bot,
    msg: Message,
    dialogue: handlers::OrigaDialogue,
    (mode, _card_ids, _current_index, _showing_answer, _new_count, _review_count): (
        LessonMode,
        Vec<ulid::Ulid>,
        usize,
        bool,
        usize,
        usize,
    ),
) -> ResponseResult<()> {
    let telegram_id = msg.chat.id.0 as u64;
    let username = msg
        .from
        .as_ref()
        .map(|u| u.first_name.as_str())
        .unwrap_or("User");

    let provider = OrigaServiceProvider::instance().await;
    let session = provider
        .get_or_create_session(telegram_id, username)
        .await?;

    match mode {
        LessonMode::Lesson => start_lesson(bot, msg, dialogue, session).await?,
        LessonMode::Fixation => start_fixation(bot, msg, dialogue, session).await?,
    }
    respond(())
}

async fn profile_endpoint(
    bot: Bot,
    msg: Message,
    current_view: dialogue::ProfileView,
) -> ResponseResult<()> {
    profile_handler(bot, msg, DialogueState::Profile { current_view }).await
}

async fn duolingo_connect_endpoint(
    bot: Bot,
    msg: Message,
    dialogue: handlers::OrigaDialogue,
) -> ResponseResult<()> {
    handle_duolingo_token(bot, msg, dialogue).await
}

async fn vocabulary_search_endpoint(
    bot: Bot,
    msg: Message,
    dialogue: handlers::OrigaDialogue,
    (page, items_per_page, stored_query): (usize, usize, String),
) -> ResponseResult<()> {
    let telegram_id = msg.chat.id.0 as u64;
    let username = msg
        .from
        .as_ref()
        .map(|u| u.first_name.as_str())
        .unwrap_or("User");

    let provider = OrigaServiceProvider::instance().await;
    let session = provider
        .get_or_create_session(telegram_id, username)
        .await?;

    let search_query = msg.text().unwrap_or(&stored_query);

    handle_vocabulary_search(
        &bot,
        msg.chat.id,
        &dialogue,
        session,
        search_query,
        page,
        items_per_page,
    )
    .await?;

    respond(())
}
