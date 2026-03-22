use crate::storage::Storage;
use orzklv::telegram::topic::Topics;
use teloxide::{payloads::SendMessageSetters, prelude::*, types::ParseMode};

pub async fn command(bot: &Bot, msg: &Message, mut db: Storage) -> ResponseResult<()> {
    let admins = match db.admin_ids() {
        Ok(a) => a,
        Err(e) => {
            bot.send_message_tf(msg.chat.id, "Can't fetch current acting admins?!", msg)
                .parse_mode(ParseMode::Html)
                .await?;

            return Ok(());
        }
    };

    match &msg.from {
        Some(u) => {
            if !admins.contains(&u.id) {
                bot.send_message_tf(
                    msg.chat.id,
                    "I don't think you're elligible to add admins?!",
                    msg,
                )
                .parse_mode(ParseMode::Html)
                .await?;

                return Ok(());
            }
        }
        None => {
            bot.send_message_tf(msg.chat.id, "Are you even real?!", msg)
                .parse_mode(ParseMode::Html)
                .await?;

            return Ok(());
        }
    };

    let text = match msg.text() {
        Some(txt) => txt,
        None => {
            bot.send_message_tf(msg.chat.id, "I don't think you've written an id?!", msg)
                .parse_mode(ParseMode::Html)
                .await?;

            return Ok(());
        }
    }
    .replace("/add", "")
    .trim()
    .parse::<u64>();

    let user = match text {
        Ok(u) => u,
        Err(_) => {
            bot.send_message_tf(
                msg.chat.id,
                "I don't think you've written a valid id?!",
                msg,
            )
            .parse_mode(ParseMode::Html)
            .await?;

            return Ok(());
        }
    };

    match db.add_admin(UserId(user)) {
        Ok(u) => {
            bot.send_message_tf(msg.chat.id, "Added the user into the admins, aight...", msg)
                .parse_mode(ParseMode::Html)
                .await?;
        }
        Err(e) => {
            eprintln!("{e}");

            bot.send_message_tf(msg.chat.id, e.to_string(), msg)
                .parse_mode(ParseMode::Html)
                .await?;
        }
    };

    Ok(())
}
