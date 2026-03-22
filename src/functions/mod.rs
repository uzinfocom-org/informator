pub mod add;
pub mod help;
pub mod start;

use crate::{bot::Command, error::Insult, storage::Storage};
// use orzklv::telegram::topic::Topics;
use teloxide::{prelude::*, types::*};

pub async fn commands(bot: Bot, me: Me, msg: Message, cmd: Command, db: Storage) -> Insult<()> {
    let _ = match cmd {
        Command::Start => crate::functions::start::command(&bot, &msg, db).await,
        Command::Help => crate::functions::help::command(&bot, &msg, &cmd).await,
        Command::Add => crate::functions::add::command(&bot, &msg, db).await,
    };

    Ok(())
}

pub async fn announcements(bot: Bot, me: Me, msg: Message, db: Storage) -> Insult<()> {
    println!("There's a post in channel");

    for chat in db.admin_chats()? {
        bot.send_message(chat, "There's been a post in channel")
            .parse_mode(ParseMode::Html)
            .await?;
    }

    Ok(())
}
