pub mod help;
pub mod start;
pub mod sync;

use crate::{bot::Command, error::Insult, storage::Storage};
use teloxide::{prelude::*, types::*};

pub async fn commands(bot: Bot, me: Me, msg: Message, cmd: Command, db: Storage) -> Insult<()> {
    let _ = match cmd {
        Command::Start => crate::functions::start::command(&bot, &msg).await,
        Command::Help => crate::functions::help::command(&bot, &msg, &cmd).await,
        Command::Sync => crate::functions::sync::command(&bot, &msg, db).await,
    };

    Ok(())
}

pub async fn announcements(bot: Bot, me: Me, msg: Message, mut db: Storage) -> Insult<()> {
    db.sync()?;

    println!("Hewwoooo");

    Ok(())
}
