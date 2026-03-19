use crate::{error::Insane, functions};
use teloxide::{
    dispatching::{UpdateFilterExt, UpdateHandler},
    prelude::*,
    utils::command::BotCommands,
};

#[derive(BotCommands, Clone, Debug)]
#[command(rename_rule = "lowercase", parse_with = "split")]
#[command(description = "These are the commands that I can understand:")]
pub enum Command {
    /// List existing commands
    Help,

    /// Starting point of the bot
    Start,

    /// Synchronize admins with cache
    Sync,
}

pub fn handler() -> UpdateHandler<Insane> {
    dptree::entry()
        // Commands
        .branch(
            Update::filter_message()
                .filter_command::<Command>()
                .endpoint(functions::commands),
        )
        .branch(Update::filter_channel_post().endpoint(functions::announcements))
}

pub fn dispatch(
    bot: &Bot,
    deps: DependencyMap,
) -> Dispatcher<Bot, Insane, teloxide::dispatching::DefaultKey> {
    Dispatcher::builder(bot.clone(), handler())
        .dependencies(deps) // dptree::deps![topics, pkgs]
        // If no handler succeeded to handle an update, this closure will be called
        .default_handler(|upd| async move {
            log::warn!("Unhandled update: {:?}", upd);
        })
        // If the dispatcher fails for some reason, execute this handler
        .error_handler(LoggingErrorHandler::with_custom_text(
            "An error has occurred in the dispatcher",
        ))
        .enable_ctrlc_handler()
        .build()
}
