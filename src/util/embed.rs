use hextool::Convert;
use serenity::all::{Color, Context, CreateEmbed, CreateEmbedAuthor, Timestamp};

use crate::env::COLOR;

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

// pub fn global_chat_embed(msg: Message) {
// TODO
// }
