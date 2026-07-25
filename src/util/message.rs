use serenity::all::{GuildId, Message};

pub fn message_url(msg: &Message) -> String {
    format!(
        "https://discord.com/channels/{}/{}/{}",
        msg.guild_id.unwrap_or(GuildId::default()),
        msg.channel_id,
        msg.id,
    )
}
