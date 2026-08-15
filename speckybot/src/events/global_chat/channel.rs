use serenity::all::{Context, CreateMessage, GuildChannel, GuildId, Ready};

use crate::events::global_chat::{check_gc_topic, gc_channels, strings::global_chat_rules};

pub async fn on_ready(ctx: &Context, ready: &Ready) {
    update_globalchat_channels(ctx, ready.guilds.iter().map(|g| g.id)).await;
}

pub async fn on_channel_update(ctx: &Context, old: Option<&GuildChannel>, new: &GuildChannel) {
    let announce_and_add = async || {
        let gc_channels = gc_channels();
        let _ = new.send_message(&ctx.http, CreateMessage::new().embed(global_chat_rules(gc_channels.len()))).await;
        gc_channels.insert(new.id);
    };

    if let Some(old) = old {
        match (old.topic.as_ref().and_then(|t| check_gc_topic(t)), new.topic.as_ref().and_then(|t| check_gc_topic(t))) {
            (None, Some(_gc)) => announce_and_add().await,
            (Some(old_gc), Some(new_gc)) if old_gc != new_gc => announce_and_add().await,
            _ => {}
        }
    }

    update_globalchat_channels(ctx, ctx.cache.guilds().into_iter()).await;
}

pub async fn update_globalchat_channels(ctx: &Context, guilds: impl Iterator<Item = GuildId>) {
    let set = gc_channels();
    set.clear();

    for guild in guilds {
        if let Ok(channels) = guild.channels(&ctx.http).await {
            for (_, channel) in channels {
                channel.topic
                .and_then(|topic| check_gc_topic(&topic))
                .filter(|gc| channel.nsfw == gc.nsfw)
                .inspect(|_| { set.insert(channel.id); });
            }
        }
    }
}
