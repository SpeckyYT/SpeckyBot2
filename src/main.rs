use serenity::all::GuildChannel;
use serenity::async_trait;
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::prelude::*;

use crate::commands::ParsedCommandData;
use crate::env::PREFIX;

pub mod env;
pub mod loader;
pub mod events;
pub mod commands;
pub mod util;

#[derive(Default)]
struct Bot {
}

#[async_trait]
impl EventHandler for Bot {
    async fn ready(&self, ctx: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);

        tokio::join!(
            events::global_chat::on_ready(&ctx, &ready),
        );
    }
    async fn message(&self, ctx: Context, msg: Message) {
        if msg.author.bot { return }

        tokio::join!(
            events::commands::on_message(&ctx, &msg),
            events::global_chat::on_message(&ctx, &msg),
        );
    }
    async fn channel_update(&self, ctx: Context, old: Option<GuildChannel>, new: GuildChannel) {
        tokio::join!(
            events::global_chat::on_channel_update(&ctx, old.as_ref(), &new),
        );
    }
}

#[tokio::main]
async fn main() {
    let _ = env::DOTENV.as_ref();

    loader::load();

    let bot = Bot::default();

    // Set gateway intents, which decides what events the bot will be notified about
    let intents = GatewayIntents::GUILD_MESSAGES | GatewayIntents::MESSAGE_CONTENT | GatewayIntents::GUILDS;

    let mut client = Client::builder(&*env::TOKEN, intents)
        .event_handler(bot)
        .await
        .expect("Err creating client");

    client.start().await.expect("Client errored");
}
