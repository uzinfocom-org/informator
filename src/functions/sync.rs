use crate::{error::Error, storage::Storage};
use orzklv::telegram::topic::Topics;
use teloxide::{payloads::SendMessageSetters, prelude::*, types::ParseMode};

fn success(users: &[UserId]) -> String {
    let users = users
        .iter()
        .map(|u| format!("- {}", u))
        .collect::<Vec<String>>()
        .join("\n");

    println!("Users: {users}");

    format!("Alright, successfully updated admins list!\n\n{}", users)
}

fn error(err: Error) -> String {
    "Nope, we fucked up somewhere, check logs for more...".to_string()
}

pub async fn command(bot: &Bot, msg: &Message, mut db: Storage) -> ResponseResult<()> {
    match db.sync() {
        Ok(u) => {
            bot.send_message_tf(msg.chat.id, success(u), msg)
                .parse_mode(ParseMode::Html)
                .await?;
        }
        Err(e) => {
            eprintln!("{e}");

            bot.send_message_tf(msg.chat.id, error(e), msg)
                .parse_mode(ParseMode::Html)
                .await?;
        }
    };

    Ok(())
}
