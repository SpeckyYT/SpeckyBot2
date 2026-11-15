use std::{sync::{LazyLock, OnceLock}, time::Instant};

use serenity::prelude::*;

pub static PROCESS_STARTED_AT: OnceLock<Instant> = OnceLock::new();
pub static BOT_CONNECTED_AT: OnceLock<Instant> = OnceLock::new();

#[path="../build.rs"]
pub mod build;

pub mod env;
pub mod db;
pub mod loader;
pub mod events;
pub mod commands;
pub mod util;

#[derive(Default)]
struct Bot {
}

#[tokio::main]
async fn main() {
    PROCESS_STARTED_AT.get_or_init(Instant::now);
    LazyLock::force(&env::DOTENV);

    loader::load();

    let bot = Bot::default();

    // Set gateway intents, which decides what events the bot will be notified about
    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT
        | GatewayIntents::GUILDS
        | GatewayIntents::GUILD_MESSAGE_TYPING
        | GatewayIntents::GUILD_MESSAGE_REACTIONS;

    let mut client = Client::builder(&*env::TOKEN, intents)
        .event_handler(bot)
        .await
        .expect("Err creating client");

    client.start().await.expect("Client errored");
}
