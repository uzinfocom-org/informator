use clap::Parser;
use informator::bot::dispatch;
use informator::clog;
use informator::config::{Config, Field};
use informator::database::Database;
use informator::{Cli, Commands};
use std::error::Error;
use std::sync::Arc;
use teloxide::{prelude::*, update_listeners::webhooks};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Starter packs
    pretty_env_logger::init();
    log::info!("Starting Bot: {}", env!("CARGO_PKG_NAME"));

    // Global instances
    let mut config = Config::default();

    // Database instance
    let database = Arc::new(
        Database::new()
            .connect("informator.db")
            .await? // are we connected?
            .bootstrap()
            .await? // are we good to get/set data?
            .build()?,
    );

    // Dependencies
    let deps = dptree::deps![database];

    // Args
    let args = Cli::parse();

    match args.command {
        Commands::Polling { token } => {
            match config.read(token, Field::Token) {
                Ok(_) => clog("Config", "Successfully read the token variable"),
                Err(e) => panic!("{}", e),
            };

            let bot = Bot::new(config.token);
            let mut dispatcher = dispatch(&bot, deps);

            clog("Mode", "starting polling on localhost");
            dispatcher.dispatch().await;

            Ok(())
        }
        Commands::Webhook {
            token,
            domain,
            port,
        } => {
            match config.read(token, Field::Token) {
                Ok(_) => clog("Config", "Successfully read the token variable"),
                Err(e) => panic!("{}", e),
            };

            match config.set(format!("https://{}", domain), Field::Domain) {
                Ok(_) => clog("Config", "Successfully set the domain variable"),
                Err(e) => panic!("{}", e),
            }

            let bot = Bot::new(config.token);
            let mut dispatcher = dispatch(&bot, deps);

            let addr = ([127, 0, 0, 1], port.unwrap_or(8445)).into(); // port 8445
            let listener = webhooks::axum(
                bot,
                webhooks::Options::new(addr, config.domain.parse().unwrap()),
            )
            .await
            .expect("Couldn't setup webhook");

            dispatcher
                .dispatch_with_listener(
                    listener,
                    LoggingErrorHandler::with_custom_text(
                        "An error has occurred in the dispatcher",
                    ),
                )
                .await;

            Ok(())
        }
        Commands::Env => {
            let bot = Bot::from_env();
            let mut dispatcher = dispatch(&bot, deps);

            match std::env::var("WEBHOOK_URL") {
                Ok(v) => {
                    clog("Mode", &format!("starting webhook on {}", v));

                    let port: u16 = std::env::var("PORT")
                        .unwrap_or("8445".to_string())
                        .parse()
                        .unwrap_or(8445);

                    let addr = ([0, 0, 0, 0], port).into();

                    let listener =
                        webhooks::axum(bot, webhooks::Options::new(addr, v.parse().unwrap()))
                            .await
                            .expect("Couldn't setup webhook");

                    dispatcher
                        .dispatch_with_listener(
                            listener,
                            LoggingErrorHandler::with_custom_text(
                                "An error has occurred in the dispatcher",
                            ),
                        )
                        .await;
                }
                Err(_) => {
                    clog("Mode", "starting polling on localhost");
                    dispatcher.dispatch().await;
                }
            }

            Ok(())
        }
    }
}
