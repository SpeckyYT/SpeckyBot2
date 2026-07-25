use hextool::Convert;
use serenity::all::{CacheHttp, Color, Context, CreateEmbed, CreateEmbedAuthor, CreateEmbedFooter, CreateMessage, Message, Timestamp};

use crate::{env::COLOR, util::{bot_user::user_avatar_url, message::message_url}};

pub fn default_embed(ctx: Option<&Context>) -> CreateEmbed {
    let mut embed = CreateEmbed::new()
        .color(Color::new(hextool::UnHex::convert(&COLOR, true, false).parse().unwrap_or(0)))
        .timestamp(Timestamp::now());

    if let Some(ctx) = ctx {
        let bot_user = ctx.cache.current_user();

        let mut author = CreateEmbedAuthor::new(&bot_user.name)
            .url("https://github.com/SpeckyYT/SpeckyBot");

        if let Some(avatar) = bot_user.avatar_url() {
            embed = embed.thumbnail(&avatar);
            author = author.icon_url(&avatar);
        }

        embed = embed.author(author);
    }

    embed
}

#[inline]
pub fn error_embed() -> CreateEmbed {
    CreateEmbed::new()
    .title("ERROR!")
    .color(Color::from_rgb(255, 0, 0))
}

pub fn global_chat_embed(ctx: &Context, msg: &Message) -> CreateMessage {
    let guild = ctx.cache().expect("Always valid").guild(msg.guild_id.unwrap_or(Default::default()));

    let mut embed = CreateEmbed::new()
    .author(CreateEmbedAuthor::new(msg.author.display_name()).icon_url(user_avatar_url(&msg.author)).url(message_url(msg)))
    .description(&msg.content)
    .timestamp(msg.timestamp);

    if let Some(guild) = guild {
        embed = embed.footer(CreateEmbedFooter::new(&guild.name).icon_url(guild.icon_url().unwrap_or_default()));
        
        if let Some(member) = &msg.member {
            if let Some(role) = member.roles.iter().find_map(|r| guild.roles.get(r).filter(|r| r.colour == Color::default())) {
                embed = embed.color(role.colour);
            }
        }
    }

    CreateMessage::new()
    .embed(embed)
    // .add_files(msg.attachments.iter().map(|att| CreateAttachment::)) // TODO: attachments?
}
