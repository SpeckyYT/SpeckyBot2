// TODO: either add some macro system to add events really easily, or cry about it

use serenity::{all::{ChannelId, Context, EventHandler, GuildChannel, GuildId, Message, MessageId, MessageUpdateEvent, Reaction, Ready, TypingStartEvent}, async_trait};

use crate::{Bot, events, util::log::log_event};

pub mod commands;
pub mod global_chat;

#[async_trait]
impl EventHandler for Bot {
    async fn ready(&self, ctx: Context, ready: Ready) {
        log_event("ready", format!("Bot connected as {}", ready.user.name));

        tokio::join!(
            events::global_chat::channel::on_ready(&ctx, &ready),
        );
    }

    async fn message(&self, ctx: Context, msg: Message) {
        if msg.author.bot { return }
        log_event("message", format!("Message {} in channel {} by {}", msg.id, msg.channel_id, msg.author.name));

        tokio::join!(
            events::commands::on_message(&ctx, &msg),
            events::global_chat::message::message(&ctx, &msg),
        );
    }

    async fn message_update(&self, ctx: Context, old: Option<Message>, new: Option<Message>, event: MessageUpdateEvent) {
        let new = match new {
            Some(new) => new,
            None => {
                if let Ok(new) = event.channel_id.message(&ctx.http, event.id).await {
                    new
                } else {
                    return
                }
            },
        };
        let author = old.as_ref().or(Some(&new)).map(|m| &m.author).or(event.author.as_ref());
        if let Some(author) = author && author.bot { return }

        log_event("message_update", format!("Message {} updated in channel {} (author: {})", event.id, event.channel_id, author.map(|a| a.name.clone()).unwrap_or("<unknown>".into())));

        tokio::join!(
            events::global_chat::message::message_update(&ctx, old.as_ref(), &new, &event),
        );
    }

    async fn message_delete(&self, ctx: Context, channel_id: ChannelId, deleted_message_id: MessageId, guild_id: Option<GuildId>) {
        log_event("message_delete", format!("Message {} deleted in channel {} (guild: {:?})", deleted_message_id, channel_id, guild_id));

        tokio::join!(
            events::global_chat::message::message_delete(&ctx, Some(deleted_message_id).into_iter()),
        );
    }

    async fn message_delete_bulk(&self, ctx: Context, channel_id: ChannelId, ids: Vec<MessageId>, guild_id: Option<GuildId>) {
        log_event("message_delete_bulk", format!("{} messages deleted in channel {} (guild: {:?})", ids.len(), channel_id, guild_id));

        tokio::join!(
            events::global_chat::message::message_delete(&ctx, ids.iter().copied()),
        );
    }

    async fn channel_update(&self, ctx: Context, old: Option<GuildChannel>, new: GuildChannel) {
        log_event("channel_update", format!("Channel {} updated (topic: {:?} → {:?})", new.id, old.as_ref().and_then(|c| c.topic.clone()), new.topic.clone()));

        tokio::join!(
            events::global_chat::channel::on_channel_update(&ctx, old.as_ref(), &new),
        );
    }

    async fn reaction_add(&self, ctx: Context, add_reaction: Reaction) {        
        if let Ok(user) = add_reaction.user(&ctx.http).await && user.bot {
            return
        }
        
        log_event("reaction_add", add_reaction.emoji.to_string());

        tokio::join!(
            events::global_chat::reaction::reaction_add(&ctx, &add_reaction),
        );
    }

    async fn typing_start(&self, ctx: Context, event: TypingStartEvent) {
        if let Ok(user) = event.user_id.to_user(&ctx.http).await && user.bot {
            return
        }

        tokio::join!(
            events::global_chat::typing::typing_start(&ctx, &event),
        );
    }
}
