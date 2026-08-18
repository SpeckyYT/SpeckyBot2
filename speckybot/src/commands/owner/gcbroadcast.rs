use futures::future::join_all;
use serenity::all::{Color, CreateEmbedAuthor, CreateMessage};

use crate::{commands::OWNER_CATEGORY, events::global_chat::{GCMessage, GCMessageTree, gc_channels, gc_messages}, util::{bot_user::bot_avatar_url, embed::{default_embed, global_chat_embed}}};

crate::command! {
    names: ["gcbroadcast","gcb"],
    category: OWNER_CATEGORY,
    run: |ctx, msg, data| {
        let gc_channels = gc_channels();
        let gc_messages = gc_messages();

        let bot_avatar_url = bot_avatar_url(ctx);

        let embed = global_chat_embed(ctx, msg).await
            .thumbnail(&bot_avatar_url)
            .title("Broadcast!")
            .author(CreateEmbedAuthor::new(&ctx.cache.current_user().name).icon_url(&bot_avatar_url))
            .description(&data.cmd_content)
            .color(Color::from_rgb(255, 0, 170));

        let messages = gc_channels
            .iter()
            .map(|c| c.send_message(&ctx.http, CreateMessage::new().embed(embed.clone())));
        
        let messages = join_all(messages).await;

        let children = messages.iter().flatten().map(|m| {
            let gcm = GCMessage {
                message: m.clone(),
                channel_id: m.channel_id,
                guild_id: m.guild_id.unwrap_or_default(),
                tree: GCMessageTree::Child(msg.id),
            };
            gc_messages.insert(m.id, gcm.clone());
            gcm
        })
        .collect();

        let gcm = GCMessage {
            message: msg.clone(),
            channel_id: msg.channel_id,
            guild_id: msg.guild_id.unwrap_or_default(),
            tree: GCMessageTree::Parent(children),
        };
        gc_messages.insert(msg.id, gcm);

        let total_amount = messages.len();
        let success_amount = messages.iter().flatten().count();
        let content = format!("Broadcasts sent `{success_amount}` (ouf of `{total_amount}`)");

        msg.channel_id.send_message(&ctx.http, CreateMessage::new().embed(default_embed(Some(ctx)).color(Color::DARK_GREEN).description(content))).await?;

        Ok(())
    }
}
