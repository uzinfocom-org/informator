pub mod help;
pub mod start;

use crate::{bot::Command, database::Database, error::Insult};
use std::sync::Arc;
use teloxide::{prelude::*, types::*};

pub async fn commands(bot: Bot, me: Me, msg: Message, cmd: Command) -> Insult<()> {
    let _ = match cmd {
        Command::Start => crate::functions::start::command(&bot, &msg).await,
        Command::Help => crate::functions::help::command(&bot, &msg, &cmd).await,
    };

    Ok(())
}

pub async fn announcements(bot: Bot, me: Me, msg: Message, database: Arc<Database>) -> Insult<()> {
    Ok(())
}
