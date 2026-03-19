use clap::Parser;
use informator::bot::dispatch;
use informator::clog;
use informator::config::{Config, Field};
use informator::error::Result;
use informator::storage::Storage;
use informator::{Cli, Commands};
use std::sync::Arc;
use teloxide::{prelude::*, update_listeners::webhooks};

#[tokio::main]
async fn main() -> Result<()> {
    match executor_task().await {
        Ok(_) => {}
        Err(e) => {
            eprintln!("error: {}", e);
            std::process::exit(1)
        }
    }

    Ok(())
}

async fn executor_task() -> Result<()> {
    // Starter packs
    pretty_env_logger::init();
    log::info!("Starting Bot: {}", env!("CARGO_PKG_NAME"));

    // Global instances
    let mut config = Config::default();

    // Args
    let args = Cli::parse();

    match args.command {
        Commands::Polling { token, database } => {
            // Database instance
            let storage = Arc::new(
                Storage::new()
                    .connect(database.to_str())
                    .await?
                    .migrate()
                    .await?
                    .sync()
                    .await?
                    .build()?,
            );

            // Dependencies
            let deps = dptree::deps![storage];

            // Bot instance
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
            database,
            domain,
            port,
        } => {
            // Database instance
            let storage = Arc::new(
                Storage::new()
                    .connect(database.to_str())
                    .await?
                    .migrate()
                    .await?
                    .sync()
                    .await?
                    .build()?,
            );

            // Dependencies
            let deps = dptree::deps![storage];

            match config.read(token, Field::Token) {
                Ok(_) => clog("Config", "Successfully read the token variable"),
                Err(e) => panic!("{}", e),
            };

            match config.set(format!("https://{}", domain), Field::Domain) {
                Ok(_) => clog("Config", "Successfully set the domain variable"),
                Err(e) => panic!("{}", e),
            }

            // Bot instance
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
            // Database instance
            let storage = Arc::new(
                Storage::new()
                    .connect(None)
                    .await?
                    .migrate()
                    .await?
                    .sync()
                    .await?
                    .build()?,
            );

            // Dependencies
            let deps = dptree::deps![storage];

            // Bot instance
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
