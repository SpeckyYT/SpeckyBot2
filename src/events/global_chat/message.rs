use futures::future::join_all;
use serenity::all::{Context, GuildChannel, Message, MessageId, MessageUpdateEvent};

use crate::{events::global_chat::{GCMessage, GCMessageTree, gc_channels, gc_messages}, util::embed::{global_chat_edit_message, global_chat_message}};

impl GCMessage {
    pub fn get_channel(&self, ctx: &Context) -> Option<GuildChannel> {
        let Some(guild) = ctx.cache.guild(self.guild_id) else { return None };
        let Some(channel) = guild.channels.get(&self.channel_id) else { return None };
        Some(channel.clone())
    }
    pub async fn get_message(&self, ctx: &Context) -> Option<Message> {
        let Some(channel) = self.get_channel(ctx) else { return None };
        let Ok(message) = channel.message(&ctx.http, self.message_id).await else { return None };
        Some(message)
    }
    pub async fn send_message(&self, ctx: &Context, msg: &Message) -> Option<Message> {
        let Some(channel) = self.get_channel(ctx) else { return None };

        let m = global_chat_message(ctx, msg);

        channel.send_message(&ctx.http, m).await.ok()
    }
    pub async fn update_message(&self, ctx: &Context, msg: &Message) -> Option<()> {
        let Some(mut message_to_edit) = self.get_message(ctx).await else { return None };

        message_to_edit.edit(&ctx.http, global_chat_edit_message(ctx, msg)).await.ok()
    }
    pub async fn delete_message(&self, ctx: &Context) -> Option<()> {
        let Some(message_to_edit) = self.get_message(ctx).await else { return None };

        message_to_edit.delete(&ctx.http).await.ok()
    }
}

pub async fn on_message(ctx: &Context, msg: &Message) {
    if cfg!(debug_assertions) { return } 

    let gc_channels = gc_channels();
    
    if gc_channels.contains(&msg.channel_id) {
        let gc_messages = gc_messages();

        let messages = gc_channels.iter()
            .filter(|id| id.get() != msg.channel_id.get())
            .map(async |c| {
                let gc_cm = global_chat_message(ctx, msg);
                let Ok(m) = c.send_message(&ctx.http, gc_cm).await else { return None };
                let gcm = GCMessage {
                    message_id: m.id,
                    channel_id: m.channel_id,
                    guild_id: m.guild_id.unwrap_or_default(),
                    tree: GCMessageTree::Child(msg.id),
                };
                gc_messages.insert(m.id, gcm.clone());
                Some(gcm)
            });

        let children = join_all(messages).await
            .into_iter()
            .flatten()
            .collect();

        gc_messages.insert(msg.id, GCMessage {
            message_id: msg.id,
            channel_id: msg.channel_id,
            guild_id: msg.guild_id.unwrap_or_default(),
            tree: GCMessageTree::Parent(children),
        });
    }
}

pub async fn message_update(ctx: &Context, _old_if_available: Option<&Message>, new: Option<&Message>, event: &MessageUpdateEvent) {
    if let Some(message) = gc_messages().get(&event.id) {
        if let GCMessageTree::Parent(children) = &message.tree {
            if let Some(new) = new {
                join_all(children.iter().map(|child| child.update_message(ctx, new))).await;
            }
        }
    }
}

pub async fn message_delete(ctx: &Context, ids: impl Iterator<Item = MessageId>) {
    join_all(
        ids
        .filter_map(|id| gc_messages().get(&id))
        .map(async |gcm| {
            match &gcm.tree {
                GCMessageTree::Parent(children) => {
                    join_all(children.iter().map(|c| c.delete_message(ctx))).await;
                },
                GCMessageTree::Child(_) => {}, // don't care if a child got deleted
            }
        })
    ).await;
}
