pub mod help;
pub mod start;

use std::sync::Arc;

use crate::{bot::Command, error::Insult, storage::Storage};
use teloxide::{prelude::*, types::*};

pub async fn commands(bot: Bot, me: Me, msg: Message, cmd: Command) -> Insult<()> {
    let _ = match cmd {
        Command::Start => crate::functions::start::command(&bot, &msg).await,
        Command::Help => crate::functions::help::command(&bot, &msg, &cmd).await,
    };

    Ok(())
}

pub async fn announcements(bot: Bot, me: Me, msg: Message, db: Arc<Storage>) -> Insult<()> {
    Ok(())
}
