use serenity::async_trait;
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::prelude::*;
use tokio;

use crate::commands::ParsedCommandData;
use crate::env::PREFIX;

mod env;
mod events;
mod commands;
mod util;

struct Bot;

#[async_trait]
impl EventHandler for Bot {
    async fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }
    async fn message(&self, ctx: Context, msg: Message) {
        tokio::join!(
            events::commands::on_message(&ctx, &msg),
        );
    }
}

#[tokio::main]
async fn main() {
    let _ = env::DOTENV.as_ref();

    // Set gateway intents, which decides what events the bot will be notified about
    let intents = GatewayIntents::GUILD_MESSAGES | GatewayIntents::MESSAGE_CONTENT;

    let mut client = Client::builder(&*env::TOKEN, intents)
        .event_handler(Bot)
        .await
        .expect("Err creating client");

    client.start().await.expect("Client errored");
}
