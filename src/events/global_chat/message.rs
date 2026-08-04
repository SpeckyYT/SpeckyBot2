use futures::future::join_all;
use itertools::Itertools;
use serenity::all::{Context, CreateAttachment, GuildChannel, Message, MessageId, MessageReference, MessageReferenceKind, MessageUpdateEvent};

use crate::{env::GLOBAL_CHAT, events::global_chat::{EDIT_LOCK, GCMessage, GCMessageTree, gc_channels, gc_messages}, util::{embed::{global_chat_edit_message, global_chat_message}, log::log_event}};

impl GCMessage {
    pub fn get_channel(&self, ctx: &Context) -> Option<GuildChannel> {
        let guild = ctx.cache.guild(self.guild_id)?;
        let channel = guild.channels.get(&self.channel_id)?;
        Some(channel.clone())
    }
    pub fn get_family(&self) -> Option<(GCMessage, Vec<GCMessage>)> {
        match &self.tree {
            GCMessageTree::Parent(children) => Some((self.clone(), children.clone())),
            GCMessageTree::Child(parent) =>
                gc_messages()
                .get(parent)
                .and_then(|p| match &p.tree {
                    GCMessageTree::Parent(children) => Some((p.clone(), children.clone())),
                    GCMessageTree::Child(_) => None, // unreachable
                })
        }
    }
    pub fn get_flat_family(&self) -> Option<Vec<GCMessage>> {
        self.get_family()
        .map(|(parent, mut children)| {
            children.push(parent);
            children
        })
    }
    pub async fn update_message(&self, ctx: &Context, msg: &Message) -> Option<()> {
        log_event("gc_update", format!("Message {} getting updated", msg.id));
        match &self.tree {
            GCMessageTree::Child(_) => {
                log_event("gc_update", format!("Message {} is a child", msg.id));
                let mut message_to_edit = self.message.clone();
                log_event("gc_update", format!("Message {} (parent {}) optained", message_to_edit.id, msg.id));
                message_to_edit.edit(&ctx.http, global_chat_edit_message(ctx, msg).await).await.ok()
            }
            GCMessageTree::Parent(_) => None,
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
        let edit_lock = EDIT_LOCK.lock();

        let gc_messages = gc_messages();

        let reference_family = msg.referenced_message.as_ref()
        .and_then(|referenced_message| gc_messages.get(&referenced_message.id))
        .and_then(|reference| reference.get_flat_family());

        let attachments = join_all(msg.attachments.iter().map(|attachment| CreateAttachment::url(&ctx.http, &attachment.url))).await;
        let attachments = attachments.iter().flatten();

        let messages = gc_channels.iter()
            .filter(|id| id.get() != msg.channel_id.get())
            .map(async |c| {
                let mut gc_cm = global_chat_message(ctx, msg).await;

                let reference = reference_family
                    .as_ref()
                    .and_then(|reference_family| reference_family.iter().find(|gcm| gcm.channel_id.get() == c.get()));

                if let Some(reference) = reference  {
                    let msg_reference = MessageReference::new(MessageReferenceKind::Default, *c)
                        .message_id(reference.message.id)
                        .fail_if_not_exists(false);
                    gc_cm = gc_cm.reference_message(msg_reference);
                }

                gc_cm = gc_cm.add_files(attachments.clone().cloned());

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

        drop(edit_lock);
    }
}

pub async fn message_update(ctx: &Context, _old_if_available: Option<&Message>, new: &Message, event: &MessageUpdateEvent) {
    if let Some(message) = gc_messages().get(&event.id) {
        log_event("gc_update", format!("Obtained GC Message {}", message.message.id));
        if let GCMessageTree::Parent(children) = &message.tree {
            let edit_lock = EDIT_LOCK.lock();
            let update_status = join_all(children.iter().map(|child| child.update_message(ctx, new))).await;
            log_event("gc_update", format!("Update statuses on message {} ({:?})", message.message.id, update_status));
            drop(edit_lock);
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
