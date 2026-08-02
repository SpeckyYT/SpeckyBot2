use std::sync::OnceLock;

use dashmap::{DashMap, DashSet};
use serenity::all::{ChannelId, GuildId, Message, MessageId};

pub mod strings;
pub mod message;
pub mod channel;

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
#[inline]
fn gc_channels() -> &'static DashSet<ChannelId> {
    GLOBAL_CHAT_CHANNELS.get_or_init(|| DashSet::new())
}

#[derive(Debug, Clone)]
pub enum GCMessageTree {
    /// Children ids
    Parent(Vec<GCMessage>),
    /// Parent id
    Child(MessageId),
}

#[derive(Debug, Clone)]
pub struct GCMessage {
    pub message: Message,
    pub channel_id: ChannelId,
    pub guild_id: GuildId,
    pub tree: GCMessageTree,
}

pub static GLOBAL_CHAT_MESSAGES: OnceLock<DashMap<MessageId, GCMessage>> = OnceLock::new();
#[inline]
fn gc_messages() -> &'static DashMap<MessageId, GCMessage> {
    GLOBAL_CHAT_MESSAGES.get_or_init(|| DashMap::new())
}

pub fn check_gc_topic(topic: &str) -> Option<&'static GloablChat> {
    GLOBAL_CHATS.iter().find(|gc| topic.contains(gc.name))
}
