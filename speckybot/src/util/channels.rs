use serenity::all::{ChannelId, Context, GuildId, Message};

pub const SPECKY_PROJECTS_GUILD: GuildId = GuildId::new(538028973058424832);
pub const COMMAND_ERRORS_CHANNEL: ChannelId = ChannelId::new(764555141280956426);

/// defaults to `false` if guild or channel wasn't found
#[inline]
pub fn is_nsfw_channel(ctx: &Context, msg: &Message) -> bool {
    msg.guild(&ctx.cache)
        .as_ref()
        .and_then(|c| c.channels.get(&msg.channel_id))
        .map(|c| c.nsfw)
        .unwrap_or(false)
}
