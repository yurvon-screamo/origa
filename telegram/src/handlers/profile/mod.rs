mod callbacks;

pub use callbacks::ProfileCallback;

use crate::dialogue::ProfileView;
use crate::handlers::OrigaDialogue;
use crate::handlers::callbacks::CallbackData;
use crate::service::OrigaServiceProvider;
use origa::application::{UserProfile, UserRepository};
use origa::domain::JapaneseLevel;
use teloxide::prelude::*;
use teloxide::types::{ChatId, InlineKeyboardButton, InlineKeyboardMarkup};
use ulid::Ulid;

pub async fn profile_handler(
    bot: Bot,
    msg: Message,
    state: crate::dialogue::DialogueState,
) -> ResponseResult<()> {
    let telegram_id = msg.chat.id.0 as u64;
    let provider = OrigaServiceProvider::instance().await;

    if let crate::dialogue::DialogueState::Profile { current_view } = state {
        match current_view {
            ProfileView::Main => {
                show_profile_main(&bot, msg.chat.id, telegram_id, provider).await?
            }
            ProfileView::Settings => {
                show_profile_settings(&bot, msg.chat.id, telegram_id, provider).await?
            }
            ProfileView::JlptSelect => show_jlpt_selector(&bot, msg.chat.id).await?,
        }
    }

    respond(())
}

pub async fn load_user_profile(
    provider: &'static OrigaServiceProvider,
    telegram_id: u64,
) -> ResponseResult<(UserProfile, Ulid)> {
    let session = provider
        .get_or_create_session(telegram_id, "User")
        .await
        .map_err(|_| {
            teloxide::RequestError::Io(std::sync::Arc::new(std::io::Error::other(
                "Failed to create session",
            )))
        })?;

    let use_case = provider.get_user_info_use_case();
    use_case
        .execute(session.user_id)
        .await
        .map(|p| (p, session.user_id))
        .map_err(|_| {
            teloxide::RequestError::Io(std::sync::Arc::new(std::io::Error::other(
                "Failed to load profile",
            )))
        })
}

async fn show_profile_main(
    bot: &Bot,
    chat_id: ChatId,
    telegram_id: u64,
    provider: &'static OrigaServiceProvider,
) -> ResponseResult<()> {
    let (profile, _) = match load_user_profile(provider, telegram_id).await {
        Ok(p) => p,
        Err(_) => {
            bot.send_message(chat_id, "Ошибка загрузки профиля").await?;
            return respond(());
        }
    };

    let duolingo_status = if profile.duolingo_jwt_token.is_some() {
        "Подключено ✓"
    } else {
        "Не подключено"
    };

    let text = format!(
        "👤 Профиль\n\nИмя: {}\n\nЦелевой уровень JLPT: {}\n\n🔗 Duolingo: {}",
        profile.username,
        profile.current_japanese_level.code(),
        duolingo_status
    );

    bot.send_message(chat_id, text)
        .reply_markup(profile_main_keyboard())
        .await?;

    respond(())
}

pub fn profile_main_keyboard() -> InlineKeyboardMarkup {
    let buttons: Vec<Vec<InlineKeyboardButton>> = vec![
        vec![InlineKeyboardButton::callback(
            "Изменить уровень JLPT ➡️",
            CallbackData::Profile(ProfileCallback::JlptSelect).to_json(),
        )],
        vec![InlineKeyboardButton::callback(
            "🔗 Подключить Duolingo",
            CallbackData::Profile(ProfileCallback::DuolingoConnect).to_json(),
        )],
        vec![InlineKeyboardButton::callback(
            "⚙️ Настройки",
            CallbackData::Profile(ProfileCallback::Settings).to_json(),
        )],
        vec![InlineKeyboardButton::callback(
            "🚪 Выйти",
            CallbackData::Profile(ProfileCallback::Exit).to_json(),
        )],
    ];
    InlineKeyboardMarkup::new(buttons)
}

async fn show_profile_settings(
    bot: &Bot,
    chat_id: ChatId,
    telegram_id: u64,
    provider: &'static OrigaServiceProvider,
) -> ResponseResult<()> {
    let (profile, _) = match load_user_profile(provider, telegram_id).await {
        Ok(p) => p,
        Err(_) => {
            bot.send_message(chat_id, "Ошибка загрузки настроек")
                .await?;
            return respond(());
        }
    };

    let reminders_status = if profile.reminders_enabled {
        "Вкл"
    } else {
        "Выкл"
    };
    let text = format!("⚙️ Настройки\n\n• Напоминания: {}", reminders_status);

    bot.send_message(chat_id, text)
        .reply_markup(settings_keyboard(profile.reminders_enabled))
        .await?;

    respond(())
}

pub fn settings_keyboard(reminders_enabled: bool) -> InlineKeyboardMarkup {
    let button_text = if reminders_enabled {
        "🔔 Напоминания: Вкл"
    } else {
        "🔔 Напоминания: Выкл"
    };

    let buttons: Vec<Vec<InlineKeyboardButton>> = vec![
        vec![InlineKeyboardButton::callback(
            button_text,
            CallbackData::Profile(ProfileCallback::RemindersToggle).to_json(),
        )],
        vec![InlineKeyboardButton::callback(
            "🔙 Назад",
            CallbackData::Profile(ProfileCallback::Back).to_json(),
        )],
    ];
    InlineKeyboardMarkup::new(buttons)
}

async fn show_jlpt_selector(bot: &Bot, chat_id: ChatId) -> ResponseResult<()> {
    let text = "Выберите целевой уровень JLPT:";

    bot.send_message(chat_id, text)
        .reply_markup(jlpt_selector_keyboard())
        .await?;

    respond(())
}

fn jlpt_selector_keyboard() -> InlineKeyboardMarkup {
    let buttons: Vec<Vec<InlineKeyboardButton>> = vec![
        vec![
            InlineKeyboardButton::callback(
                "N5",
                CallbackData::Profile(ProfileCallback::JlptSet {
                    level: JapaneseLevel::N5,
                })
                .to_json(),
            ),
            InlineKeyboardButton::callback(
                "N4",
                CallbackData::Profile(ProfileCallback::JlptSet {
                    level: JapaneseLevel::N4,
                })
                .to_json(),
            ),
        ],
        vec![
            InlineKeyboardButton::callback(
                "N3",
                CallbackData::Profile(ProfileCallback::JlptSet {
                    level: JapaneseLevel::N3,
                })
                .to_json(),
            ),
            InlineKeyboardButton::callback(
                "N2",
                CallbackData::Profile(ProfileCallback::JlptSet {
                    level: JapaneseLevel::N2,
                })
                .to_json(),
            ),
        ],
        vec![InlineKeyboardButton::callback(
            "N1",
            CallbackData::Profile(ProfileCallback::JlptSet {
                level: JapaneseLevel::N1,
            })
            .to_json(),
        )],
        vec![InlineKeyboardButton::callback(
            "🔙 Назад",
            CallbackData::Profile(ProfileCallback::Back).to_json(),
        )],
    ];
    InlineKeyboardMarkup::new(buttons)
}

async fn show_duolingo_connect(
    bot: &Bot,
    chat_id: ChatId,
    dialogue: &OrigaDialogue,
) -> ResponseResult<()> {
    use crate::dialogue::DialogueState;

    let text = "🔗 Подключение Duolingo\n\nДля подключения аккаунта Duolingo, пожалуйста, отправьте ваш токен авторизации (JWT token).\n\nПолучить токен можно через инструменты разработчика в браузере.\n\nОтправьте токен в следующем сообщении:";

    let keyboard = InlineKeyboardMarkup::new(vec![vec![InlineKeyboardButton::callback(
        "🔙 Назад",
        CallbackData::Profile(ProfileCallback::Back).to_json(),
    )]]);

    bot.send_message(chat_id, text)
        .reply_markup(keyboard)
        .await?;

    dialogue
        .update(DialogueState::DuolingoConnect)
        .await
        .map_err(|e| {
            teloxide::RequestError::Io(std::sync::Arc::new(std::io::Error::other(e.to_string())))
        })?;

    respond(())
}

pub async fn profile_callback_handler(
    bot: &Bot,
    q: &CallbackQuery,
    callback: ProfileCallback,
    dialogue: &OrigaDialogue,
) -> ResponseResult<()> {
    let chat_id = q.message.as_ref().map(|m| m.chat().id);

    match callback {
        ProfileCallback::JlptSelect => {
            if let Some(chat_id) = chat_id {
                use crate::dialogue::DialogueState;
                dialogue
                    .update(DialogueState::Profile {
                        current_view: ProfileView::JlptSelect,
                    })
                    .await
                    .map_err(|e| {
                        teloxide::RequestError::Io(std::sync::Arc::new(std::io::Error::other(
                            e.to_string(),
                        )))
                    })?;
                show_jlpt_selector(bot, chat_id).await?;
            }
        }
        ProfileCallback::DuolingoConnect => {
            if let Some(chat_id) = chat_id {
                show_duolingo_connect(bot, chat_id, dialogue).await?;
            }
        }
        ProfileCallback::Settings => {
            if let Some(chat_id) = chat_id {
                use crate::dialogue::DialogueState;
                dialogue
                    .update(DialogueState::Profile {
                        current_view: ProfileView::Settings,
                    })
                    .await
                    .map_err(|e| {
                        teloxide::RequestError::Io(std::sync::Arc::new(std::io::Error::other(
                            e.to_string(),
                        )))
                    })?;
                let provider = OrigaServiceProvider::instance().await;
                show_profile_settings(bot, chat_id, chat_id.0 as u64, provider).await?;
            }
        }
        ProfileCallback::RemindersToggle => {
            if let Some(_chat_id) = chat_id {
                handle_reminders_toggle(bot, q, dialogue).await?;
            }
        }
        ProfileCallback::Exit => {
            if let Some(chat_id) = chat_id {
                handle_exit(bot, chat_id, dialogue).await?;
            }
        }
        ProfileCallback::Back => {
            if let Some(chat_id) = chat_id {
                use crate::dialogue::DialogueState;
                dialogue
                    .update(DialogueState::Profile {
                        current_view: ProfileView::Main,
                    })
                    .await
                    .map_err(|e| {
                        teloxide::RequestError::Io(std::sync::Arc::new(std::io::Error::other(
                            e.to_string(),
                        )))
                    })?;
                let telegram_id = chat_id.0 as u64;
                let provider = OrigaServiceProvider::instance().await;
                show_profile_main(bot, chat_id, telegram_id, provider).await?;
            }
        }
        ProfileCallback::JlptSet { level } => {
            handle_jlpt_selection(bot, q, level).await?;
        }
        ProfileCallback::ConfirmExit => {
            confirm_exit_handler(bot, q, dialogue).await?;
        }
    }

    respond(())
}

async fn handle_jlpt_selection(
    bot: &Bot,
    q: &CallbackQuery,
    level: JapaneseLevel,
) -> ResponseResult<()> {
    if let Some(chat_id) = q.message.as_ref().map(|m| m.chat().id) {
        let telegram_id = chat_id.0 as u64;
        let provider = OrigaServiceProvider::instance().await;

        let (current_profile, user_id) = match load_user_profile(provider, telegram_id).await {
            Ok(p) => p,
            Err(_) => {
                bot.send_message(chat_id, "Ошибка обновления уровня")
                    .await?;
                return respond(());
            }
        };

        let update_use_case = provider.update_user_profile_use_case();

        match update_use_case
            .execute(
                user_id,
                level,
                current_profile.native_language,
                current_profile.duolingo_jwt_token,
                Some(telegram_id),
                current_profile.reminders_enabled,
            )
            .await
        {
            Ok(_) => {
                bot.send_message(chat_id, format!("Уровень JLPT изменен на {}", level.code()))
                    .await?;
                show_profile_main(bot, chat_id, telegram_id, provider).await?;
            }
            Err(_) => {
                bot.send_message(chat_id, "Ошибка обновления уровня")
                    .await?;
            }
        }
    }

    respond(())
}

async fn handle_reminders_toggle(
    bot: &Bot,
    q: &CallbackQuery,
    _dialogue: &OrigaDialogue,
) -> ResponseResult<()> {
    if let Some(chat_id) = q.message.as_ref().map(|m| m.chat().id) {
        let telegram_id = chat_id.0 as u64;
        let provider = OrigaServiceProvider::instance().await;

        let (current_profile, user_id) = match load_user_profile(provider, telegram_id).await {
            Ok(p) => p,
            Err(_) => {
                bot.send_message(chat_id, "Ошибка обновления настроек")
                    .await?;
                return respond(());
            }
        };

        let new_state = !current_profile.reminders_enabled;
        let update_use_case = provider.update_user_profile_use_case();

        match update_use_case
            .execute(
                user_id,
                current_profile.current_japanese_level,
                current_profile.native_language,
                current_profile.duolingo_jwt_token,
                Some(telegram_id),
                new_state,
            )
            .await
        {
            Ok(_) => {
                let status_text = if new_state { "Вкл" } else { "Выкл" };
                bot.send_message(chat_id, format!("🔔 Напоминания: {}", status_text))
                    .await?;
                show_profile_settings(bot, chat_id, telegram_id, provider).await?;
            }
            Err(_) => {
                bot.send_message(chat_id, "Ошибка обновления настроек")
                    .await?;
            }
        }
    }

    respond(())
}

async fn handle_exit(bot: &Bot, chat_id: ChatId, _dialogue: &OrigaDialogue) -> ResponseResult<()> {
    let text = "Вы уверены, что хотите удалить все данные? Это действие нельзя отменить.";

    let keyboard = InlineKeyboardMarkup::new(vec![vec![
        InlineKeyboardButton::callback(
            "✅ Да, удалить",
            CallbackData::Profile(ProfileCallback::ConfirmExit).to_json(),
        ),
        InlineKeyboardButton::callback(
            "❌ Отмена",
            CallbackData::Profile(ProfileCallback::Back).to_json(),
        ),
    ]]);

    bot.send_message(chat_id, text)
        .reply_markup(keyboard)
        .await?;

    respond(())
}

pub async fn confirm_exit_handler(
    bot: &Bot,
    q: &CallbackQuery,
    dialogue: &OrigaDialogue,
) -> ResponseResult<()> {
    if let Some(chat_id) = q.message.as_ref().map(|m| m.chat().id) {
        let provider = OrigaServiceProvider::instance().await;
        let telegram_id = chat_id.0 as u64;

        if let Ok(Some(user)) = provider
            .repository()
            .find_by_telegram_id(&telegram_id)
            .await
        {
            match provider.repository().delete(user.id()).await {
                Ok(_) => {
                    bot.send_message(chat_id, "Ваши данные удалены. До свидания! 👋")
                        .await?;
                    dialogue.exit().await.ok();
                }
                Err(_) => {
                    bot.send_message(chat_id, "Ошибка удаления данных").await?;
                }
            }
        } else {
            bot.send_message(chat_id, "Данные не найдены").await?;
        }
    }

    respond(())
}

pub async fn handle_duolingo_token(
    bot: Bot,
    msg: Message,
    dialogue: OrigaDialogue,
) -> ResponseResult<()> {
    let token = msg.text().unwrap_or("").trim();

    if token.is_empty() || token.len() < 10 {
        bot.send_message(
            msg.chat.id,
            "❌ Неверный формат токена. Пожалуйста, отправьте валидный JWT токен.",
        )
        .await?;
        return respond(());
    }

    let telegram_id = msg.chat.id.0 as u64;
    let provider = OrigaServiceProvider::instance().await;

    let session = match provider.get_or_create_session(telegram_id, "User").await {
        Ok(s) => s,
        Err(_) => {
            bot.send_message(msg.chat.id, "❌ Ошибка загрузки профиля")
                .await?;
            return respond(());
        }
    };

    let update_use_case = provider.update_user_profile_use_case();
    let current_profile = match provider
        .get_user_info_use_case()
        .execute(session.user_id)
        .await
    {
        Ok(p) => p,
        Err(_) => {
            bot.send_message(msg.chat.id, "❌ Ошибка загрузки профиля")
                .await?;
            return respond(());
        }
    };

    match update_use_case
        .execute(
            session.user_id,
            current_profile.current_japanese_level,
            current_profile.native_language,
            Some(token.to_string()),
            Some(telegram_id),
            current_profile.reminders_enabled,
        )
        .await
    {
        Ok(_) => {
            bot.send_message(msg.chat.id, "✅ Duolingo успешно подключен!")
                .await?;
            dialogue.exit().await.ok();
        }
        Err(_) => {
            bot.send_message(msg.chat.id, "❌ Ошибка подключения Duolingo")
                .await?;
        }
    }

    respond(())
}
