use serenity::all::{ChannelId, GuildChannel, GuildId, MessageId, MessageUpdateEvent};
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
            events::global_chat::channel::on_ready(&ctx, &ready),
        );
    }
    async fn message(&self, ctx: Context, msg: Message) {
        if msg.author.bot { return }

        tokio::join!(
            events::commands::on_message(&ctx, &msg),
            events::global_chat::message::on_message(&ctx, &msg),
        );
    }
    async fn message_update(&self, ctx: Context, old_if_available: Option<Message>, new: Option<Message>, event: MessageUpdateEvent) {
        if let Some(author) = old_if_available.as_ref().or(new.as_ref()).map(|m| &m.author).or(event.author.as_ref()) {
            if author.bot { return }
        }

        tokio::join!(
            events::global_chat::message::message_update(&ctx, old_if_available.as_ref(), new.as_ref(), &event),
        );
    }
    async fn message_delete(&self, ctx: Context, _channel_id: ChannelId, deleted_message_id: MessageId, _guild_id: Option<GuildId>) {
        tokio::join!(
            events::global_chat::message::message_delete(&ctx, Some(deleted_message_id).into_iter()),
        );
    }
    async fn message_delete_bulk(&self, ctx: Context, _channel_id: ChannelId, multiple_deleted_messages_ids: Vec<MessageId>, _guild_id: Option<GuildId>) {
        tokio::join!(
            events::global_chat::message::message_delete(&ctx, multiple_deleted_messages_ids.iter().copied()),
        );
    }
    async fn channel_update(&self, ctx: Context, old: Option<GuildChannel>, new: GuildChannel) {
        tokio::join!(
            events::global_chat::channel::on_channel_update(&ctx, old.as_ref(), &new),
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
