mod bot;
mod handlers;
mod repository;
mod telegram_domain;

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
use origa::{application::UserRepository, infrastructure::FileSystemUserRepository};
use std::path::PathBuf;
use telegram_domain::{DialogueState, SessionData};
use teloxide::dispatching::dialogue::{self, InMemStorage};
use teloxide::prelude::*;
use tracing::info;

async fn build_repository() -> Result<FileSystemUserRepository, origa::domain::OrigaError> {
    let path = PathBuf::from("./data/users");
    FileSystemUserRepository::new(path).await
}

async fn find_or_create_session(
    repository: &FileSystemUserRepository,
    telegram_id: u64,
    username: &str,
) -> Result<SessionData, origa::domain::OrigaError> {
    if let Some(user) = repository.find_by_telegram_id(&telegram_id).await? {
        return Ok(SessionData {
            user_id: user.id(),
            username: username.to_string(),
        });
    }

    let user = origa::domain::User::new(
        username.to_string(),
        origa::domain::JapaneseLevel::N5,
        origa::domain::NativeLanguage::Russian,
    );
    repository.save(&user).await?;

    Ok(SessionData {
        user_id: ulid::Ulid::new(),
        username: username.to_string(),
    })
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt().init();

    let bot = Bot::new(std::env::var("TELOXIDE_TOKEN").unwrap());
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

    let handler = dialogue::enter::<Update, InMemStorage<DialogueState>, DialogueState, _>()
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
    let repository = build_repository().await.map_err(|e| {
        teloxide::RequestError::Io(std::sync::Arc::new(std::io::Error::other(e.to_string())))
    })?;

    let username = msg
        .from
        .as_ref()
        .map(|u| u.first_name.as_str())
        .unwrap_or("User");

    let session = find_or_create_session(&repository, telegram_id, username)
        .await
        .map_err(|e| {
            teloxide::RequestError::Io(std::sync::Arc::new(std::io::Error::other(e.to_string())))
        })?;

    vocabulary_list_handler(bot, msg, dialogue, (page, items_per_page, filter), session).await
}

async fn add_from_text_endpoint(
    bot: Bot,
    msg: Message,
    dialogue: handlers::OrigaDialogue,
    pending_words: Vec<String>,
) -> ResponseResult<()> {
    let telegram_id = msg.chat.id.0 as u64;
    let repository = build_repository().await.map_err(|e| {
        teloxide::RequestError::Io(std::sync::Arc::new(std::io::Error::other(e.to_string())))
    })?;

    let username = msg
        .from
        .as_ref()
        .map(|u| u.first_name.as_str())
        .unwrap_or("User");

    let session = find_or_create_session(&repository, telegram_id, username)
        .await
        .map_err(|e| {
            teloxide::RequestError::Io(std::sync::Arc::new(std::io::Error::other(e.to_string())))
        })?;

    add_from_text_handler(bot, msg, dialogue, pending_words, session).await
}

async fn kanji_endpoint(
    bot: Bot,
    msg: Message,
    dialogue: handlers::OrigaDialogue,
    (_level, page, items_per_page): (String, usize, usize),
) -> ResponseResult<()> {
    let telegram_id = msg.chat.id.0 as u64;
    let repository = build_repository().await.map_err(|e| {
        teloxide::RequestError::Io(std::sync::Arc::new(std::io::Error::other(e.to_string())))
    })?;

    let username = msg
        .from
        .as_ref()
        .map(|u| u.first_name.as_str())
        .unwrap_or("User");

    let session = find_or_create_session(&repository, telegram_id, username)
        .await
        .map_err(|e| {
            teloxide::RequestError::Io(std::sync::Arc::new(std::io::Error::other(e.to_string())))
        })?;

    handle_kanji_list(bot, msg, dialogue, (page, items_per_page), session).await
}

async fn grammar_endpoint(
    bot: Bot,
    msg: Message,
    dialogue: handlers::OrigaDialogue,
    (page, items_per_page): (usize, usize),
) -> ResponseResult<()> {
    let telegram_id = msg.chat.id.0 as u64;
    let repository = build_repository().await.map_err(|e| {
        teloxide::RequestError::Io(std::sync::Arc::new(std::io::Error::other(e.to_string())))
    })?;

    let username = msg
        .from
        .as_ref()
        .map(|u| u.first_name.as_str())
        .unwrap_or("User");

    let session = find_or_create_session(&repository, telegram_id, username)
        .await
        .map_err(|e| {
            teloxide::RequestError::Io(std::sync::Arc::new(std::io::Error::other(e.to_string())))
        })?;

    grammar_list_handler(bot, msg, dialogue, (page, items_per_page), session).await
}

async fn lesson_endpoint(
    bot: Bot,
    msg: Message,
    dialogue: handlers::OrigaDialogue,
    (mode, _card_ids, _current_index, _showing_answer, _new_count, _review_count): (
        telegram_domain::state::LessonMode,
        Vec<ulid::Ulid>,
        usize,
        bool,
        usize,
        usize,
    ),
) -> ResponseResult<()> {
    let telegram_id = msg.chat.id.0 as u64;
    let repository = build_repository().await.map_err(|e| {
        teloxide::RequestError::Io(std::sync::Arc::new(std::io::Error::other(e.to_string())))
    })?;

    let username = msg
        .from
        .as_ref()
        .map(|u| u.first_name.as_str())
        .unwrap_or("User");

    let session = find_or_create_session(&repository, telegram_id, username)
        .await
        .map_err(|e| {
            teloxide::RequestError::Io(std::sync::Arc::new(std::io::Error::other(e.to_string())))
        })?;

    match mode {
        telegram_domain::state::LessonMode::Lesson => {
            start_lesson(bot, msg, dialogue, session).await?
        }
        telegram_domain::state::LessonMode::Fixation => {
            start_fixation(bot, msg, dialogue, session).await?
        }
    }
    respond(())
}

async fn profile_endpoint(
    bot: Bot,
    msg: Message,
    dialogue: handlers::OrigaDialogue,
    current_view: String,
) -> ResponseResult<()> {
    let state = DialogueState::Profile { current_view };
    profile_handler(bot, msg, dialogue, state).await
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
    let repository = build_repository().await.map_err(|e| {
        teloxide::RequestError::Io(std::sync::Arc::new(std::io::Error::other(e.to_string())))
    })?;

    let username = msg
        .from
        .as_ref()
        .map(|u| u.first_name.as_str())
        .unwrap_or("User");

    let session = find_or_create_session(&repository, telegram_id, username)
        .await
        .map_err(|e| {
            teloxide::RequestError::Io(std::sync::Arc::new(std::io::Error::other(e.to_string())))
        })?;

    let search_query = msg.text().unwrap_or(&stored_query);

    handlers::vocabulary::search::handle_vocabulary_search(
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
