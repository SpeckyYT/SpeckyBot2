use std::sync::OnceLock;

use dashmap::DashSet;
use futures::future::join_all;
use serenity::all::{ChannelId, Context, CreateMessage, GuildChannel, GuildId, Message, Ready};

use crate::{events::global_chat::strings::global_chat_rules, util::embed::global_chat_embed};

mod strings;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GloablChat {
    pub name: &'static str,
    pub nsfw: bool,
}

const GLOBAL_CHATS: &[GloablChat] = &[
    GloablChat {
        name: "[GLOBAL]",
        nsfw: false,
    },
    // GloablChat {
    //     name: "[GLOBAL-NSFW]",
    //     nsfw: true,
    // },
];

pub static GLOBAL_CHAT_CHANNELS: OnceLock<DashSet<ChannelId>> = OnceLock::new();

fn gc_channels() -> &'static DashSet<ChannelId> {
    GLOBAL_CHAT_CHANNELS.get_or_init(|| DashSet::new())
}

pub fn check_gc_topic(topic: &str) -> Option<&'static GloablChat> {
    GLOBAL_CHATS.iter().find(|gc| topic.contains(gc.name))
}

pub async fn update_globalchat_channels(ctx: &Context, guilds: impl Iterator<Item = GuildId>) {
    let set = gc_channels();
    set.clear();

    for guild in guilds {
        if let Ok(channels) = guild.channels(&ctx.http).await {
            for (_, channel) in channels {
                if let Some(topic) = channel.topic {
                    if let Some(global_chat) = check_gc_topic(&topic) {
                        if channel.nsfw == global_chat.nsfw {
                            set.insert(channel.id); // TODO: multiple globalchats?
                        }
                    }
                }
            }
        }
    }
}

pub async fn on_ready(ctx: &Context, ready: &Ready) {
    update_globalchat_channels(ctx, ready.guilds.iter().map(|g| g.id)).await;
}

pub async fn on_channel_update(ctx: &Context, old: Option<&GuildChannel>, new: &GuildChannel) {
    if let Some(old) = old {
        match (old.topic.as_ref().and_then(|t| check_gc_topic(&t)), new.topic.as_ref().and_then(|t| check_gc_topic(&t))) {
            (None, Some(_gc)) => {
                let _ = new.send_message(&ctx.http, CreateMessage::new().embed(global_chat_rules(247))).await; // TODO: gc count
                gc_channels().insert(new.id);
            },
            (Some(old_gc), Some(new_gc)) => {
                if old_gc != new_gc {
                    let _ = new.send_message(&ctx.http, CreateMessage::new().embed(global_chat_rules(247))).await; // TODO: gc count
                    gc_channels().insert(new.id);
                }
            }
            _ => {}
        }
    }

    update_globalchat_channels(ctx, ctx.cache.guilds().into_iter()).await;
}

pub async fn on_message(ctx: &Context, msg: &Message) {
    if cfg!(debug_assertions) { return } 

    let gc_channels = gc_channels();

    if gc_channels.contains(&msg.channel_id) {
        let messages = gc_channels.iter().filter(|id| id.get() != msg.channel_id.get()).map(|c| c.send_message(&ctx.http, global_chat_embed(ctx, msg)));
        join_all(messages).await;
    }
}
