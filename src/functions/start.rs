use crate::{error::Error, storage::Storage};
use orzklv::{telegram::keyboard::Keyboard, telegram::topic::Topics};
use teloxide::{
    payloads::SendMessageSetters,
    prelude::*,
    types::{InlineKeyboardMarkup, ParseMode},
};

static TEXT_S: &str = r#"
<b>Welcome to announcer bot!</b>

This bot should supposedly help you with your announcement stuff.
"#;

static TEXT_R: &str = r#"
<b>Hello returning user!</b>

You already know the drill, so welcome back...
"#;

static TEXT_F: &str = r#"
<b>Welcome to announcer bot!</b>

This bot should supposedly help you with your announcement stuff. However, there was a trouble while signing you to our database, please try /start command later to complete signing up...
"#;

pub async fn command(bot: &Bot, msg: &Message, mut db: Storage) -> ResponseResult<()> {
    let user_id = match &msg.from {
        Some(u) => u,
        None => {
            bot.send_message_tf(
                msg.chat.id,
                "Are you even real?! You don't seem a person...",
                msg,
            )
            .parse_mode(ParseMode::Html)
            .reply_markup(keyboard())
            .await?;

            return Ok(());
        }
    };

    match db.add_user(user_id.id, msg.chat.id) {
        Ok(_) => {
            bot.send_message_tf(msg.chat.id, TEXT_S, msg)
                .parse_mode(ParseMode::Html)
                .reply_markup(keyboard())
                .await?;
        }
        Err(error) => match error {
            Error::ReturningUser => {
                bot.send_message_tf(msg.chat.id, TEXT_R, msg)
                    .parse_mode(ParseMode::Html)
                    .reply_markup(keyboard())
                    .await?;
            }
            _ => {
                bot.send_message_tf(msg.chat.id, TEXT_F, msg)
                    .parse_mode(ParseMode::Html)
                    .reply_markup(keyboard())
                    .await?;
            }
        },
    };

    Ok(())
}

pub fn keyboard() -> InlineKeyboardMarkup {
    let mut keyboard = Keyboard::new();
    keyboard
        .url(
            "Maybe read more?",
            "https://github.com/bleur-org/templates/tree/main/rust-telegram",
        )
        .unwrap()
}
