use std::time::Duration;

use serenity::all::{Context, TypingStartEvent};

use crate::{events::global_chat::gc_channels, util::a_sync::drop_later};

const TYPING_DURATION: Duration = Duration::from_secs(7);

pub async fn typing_start(ctx: &Context, event: &TypingStartEvent) {
    let gc_channels = gc_channels();
    if !gc_channels.contains(&event.channel_id) { return }

    gc_channels.iter()
    .filter(|c| c.get() != event.channel_id.get())
    .for_each(|c| drop_later(c.start_typing(&ctx.http), TYPING_DURATION));
}
