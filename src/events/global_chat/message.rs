use futures::future::join_all;
use itertools::Itertools;
use serenity::all::{Context, GuildChannel, Message, MessageId, MessageUpdateEvent};

use crate::{env::GLOBAL_CHAT, events::global_chat::{GCMessage, GCMessageTree, gc_channels, gc_messages}, util::{embed::{global_chat_edit_message, global_chat_message}, log::log_event}};

impl GCMessage {
    pub fn get_channel(&self, ctx: &Context) -> Option<GuildChannel> {
        let Some(guild) = ctx.cache.guild(self.guild_id) else { return None };
        let Some(channel) = guild.channels.get(&self.channel_id) else { return None };
        Some(channel.clone())
    }
    pub async fn update_message(&self, ctx: &Context, msg: &Message) -> Option<()> {
        log_event("gc_update", format!("Message {} getting updated", msg.id));
        match &self.tree {
            GCMessageTree::Child(_) => {
                log_event("gc_update", format!("Message {} is a child", msg.id));
                let mut message_to_edit = self.message.clone();
                log_event("gc_update", format!("Message {} (parent {}) optained", message_to_edit.id, msg.id));
                message_to_edit.edit(&ctx.http, global_chat_edit_message(ctx, msg)).await.ok()
            }
            GCMessageTree::Parent(_) => {
                return None;
            }
        }
    }
    pub async fn delete_message(&self, ctx: &Context) -> Option<()> {
        self.message.delete(&ctx.http).await.ok()
    }
}

pub async fn message(ctx: &Context, msg: &Message) {
    if !*GLOBAL_CHAT { return }

    let gc_channels = gc_channels();
    
    if gc_channels.contains(&msg.channel_id) {
        let gc_messages = gc_messages();

        let messages = gc_channels.iter()
            .filter(|id| id.get() != msg.channel_id.get())
            .map(async |c| {
                let gc_cm = global_chat_message(ctx, msg);
                let Ok(m) = c.send_message(&ctx.http, gc_cm).await else { return None };
                let gcm = GCMessage {
                    message: m.clone(),
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
            message: msg.clone(),
            channel_id: msg.channel_id,
            guild_id: msg.guild_id.unwrap_or_default(),
            tree: GCMessageTree::Parent(children),
        });
    }
}

pub async fn message_update(ctx: &Context, _old_if_available: Option<&Message>, new: &Message, event: &MessageUpdateEvent) {
    if let Some(message) = gc_messages().get(&event.id) {
        log_event("gc_update", format!("Obtained GC Message {}", message.message.id));
        if let GCMessageTree::Parent(children) = &message.tree {
            let update_status = join_all(children.iter().map(|child| child.update_message(ctx, new))).await;
            log_event("gc_update", format!("Update statuses on message {} ({:?})", message.message.id, update_status));
        }
    }
}

pub async fn message_delete(ctx: &Context, ids: impl Iterator<Item = MessageId> + Clone) {
    log_event("gc_delete", format!("Maybe deleting messages {}", ids.clone().join(",")));
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
