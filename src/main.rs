use serenity::prelude::*;

use crate::commands::ParsedCommandData;
use crate::env::PREFIX;

#[path="../build.rs"]
pub mod build;

pub mod env;
pub mod loader;
pub mod events;
pub mod commands;
pub mod util;

#[derive(Default)]
struct Bot {
}

#[tokio::main]
async fn main() {
    let _ = env::DOTENV.as_ref();

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
