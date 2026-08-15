use futures::future::join_all;
use serenity::all::{Context, Reaction};

use crate::events::global_chat::{gc_channels, gc_messages};

pub async fn reaction_add(ctx: &Context, add_reaction: &Reaction) {
    let gc_channels = gc_channels();
    if !gc_channels.contains(&add_reaction.channel_id) { return }

    let gc_messages = gc_messages();
    let Some(gcm) = gc_messages.get(&add_reaction.message_id) else { return };

    let Some(family) = gcm.get_flat_family() else { return };

    let reactions = family.iter()
    // .filter(|gc| gc.get() != add_reaction.channel_id.get())
    .map(|gcm| gcm.message.react(&ctx.http, add_reaction.emoji.clone()));

    join_all(reactions).await;
}
