use crate::dialogue::{self, DialogueState};
use teloxide::prelude::*;

pub mod add_from_text;
pub mod callbacks;
pub mod common_handlers;
pub mod extractors;
pub mod grammar;
pub mod kanji;
pub mod lesson;
pub mod main_menu;
pub mod menu;
pub mod menu_callback;
pub mod profile;
pub mod vocabulary;

pub use common_handlers::{Command, callback_handler, help_handler, start_handler};
pub use extractors::{
    chat_id_from_msg, handle_common_text, telegram_id_from_msg, username_from_msg,
};
pub use grammar::grammar_list_handler;
pub use kanji::handle_kanji_list;
pub use main_menu::main_menu_handler;
pub use profile::{handle_duolingo_token, profile_handler};
pub use vocabulary::{handle_vocabulary_search, vocabulary_list_handler};

pub use dialogue::SessionData;
pub type OrigaDialogue = teloxide::dispatching::dialogue::Dialogue<
    DialogueState,
    teloxide::dispatching::dialogue::InMemStorage<DialogueState>,
>;

pub async fn endpoint_with_common_text<F, Fut>(
    bot: Bot,
    msg: Message,
    dialogue: OrigaDialogue,
    handler: F,
) -> teloxide::requests::ResponseResult<()>
where
    F: FnOnce(Bot, Message, OrigaDialogue) -> Fut,
    Fut: std::future::Future<Output = teloxide::requests::ResponseResult<()>>,
{
    if msg.text().is_none() {
        return respond(());
    }

    if let Some(text) = msg.text() {
        let handled = handle_common_text(bot.clone(), msg.clone(), dialogue.clone(), text).await?;
        if handled {
            return respond(());
        }
    }

    handler(bot, msg, dialogue).await
}
