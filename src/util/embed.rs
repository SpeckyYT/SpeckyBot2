use hextool::Convert;
use serenity::all::{CacheHttp, Color, Context, CreateEmbed, CreateEmbedAuthor, CreateEmbedFooter, CreateMessage, EditMessage, Embed, Message, Timestamp};

use crate::{env::COLOR, util::bot_user::user_avatar_url};

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

pub fn global_chat_edit_message(ctx: &Context, msg: &Message) -> EditMessage {
    EditMessage::new().embeds(global_chat_embeds(ctx, msg))
}

pub fn global_chat_message(ctx: &Context, msg: &Message) -> CreateMessage {
    CreateMessage::new().embeds(global_chat_embeds(ctx, msg))
}

pub fn global_chat_embeds(ctx: &Context, msg: &Message) -> Vec<CreateEmbed> {
    let mut embeds = Vec::with_capacity(msg.embeds.len() + 1);
    embeds.push(global_chat_embed(ctx, msg));
    embeds.extend(msg.embeds.iter().map(|e| embed_to_create_embed(e)));
    embeds
}

pub fn global_chat_embed(ctx: &Context, msg: &Message) -> CreateEmbed {
    let guild = ctx.cache().expect("Always valid").guild(msg.guild_id.unwrap_or(Default::default()));

    let mut embed = CreateEmbed::new()
    .author(CreateEmbedAuthor::new(msg.author.display_name()).icon_url(user_avatar_url(&msg.author)).url(msg.link()))
    .description(&msg.content)
    .timestamp(msg.timestamp);

    if let Some(guild) = guild {
        embed = embed.footer(CreateEmbedFooter::new(&guild.name).icon_url(guild.icon_url().unwrap_or_default()));

        if let Some(member) = guild.members.get(&msg.author.id) {
            if let Some(color) = member.colour(&ctx.cache) {
                embed = embed.color(color);
            }
        }
    }

    embed
}

pub fn embed_to_create_embed(embed: &Embed) -> CreateEmbed {
    let mut ce = CreateEmbed::new();

    if let Some(title) = &embed.title {
        ce = ce.title(title);
    }

    if let Some(description) = &embed.description {
        ce = ce.description(description);
    }

    if let Some(url) = &embed.url {
        ce = ce.url(url);
    }

    if let Some(color) = embed.colour {
        ce = ce.color(color);
    }

    if let Some(author) = &embed.author {
        ce = ce.author({
            let mut a = CreateEmbedAuthor::new(&author.name);
            if let Some(url) = &author.url {
                a = a.url(url);
            }
            if let Some(icon) = &author.icon_url {
                a = a.icon_url(icon);
            }
            a
        });
    }

    for field in &embed.fields {
        ce = ce.field(&field.name, &field.value, field.inline);
    }

    if let Some(image) = &embed.image {
        ce = ce.image(&image.url);
    }

    if let Some(thumb) = &embed.thumbnail {
        ce = ce.thumbnail(&thumb.url);
    }

    if let Some(footer) = &embed.footer {
        ce = ce.footer({
            let mut f = CreateEmbedFooter::new(&footer.text);
            if let Some(icon) = &footer.icon_url {
                f = f.icon_url(icon);
            }
            f
        });
    }

    if let Some(timestamp) = &embed.timestamp {
        ce = ce.timestamp(timestamp);
    }

    ce
}
