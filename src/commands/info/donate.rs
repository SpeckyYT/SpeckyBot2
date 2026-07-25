use serenity::all::{CreateEmbedFooter, CreateMessage};

use crate::util::{bot_user::avatar_url, embed::default_embed};

crate::command! {
    names: ["donate","donations","donation","donator","patreon"],
    category: "info",
    run: |ctx, msg, _data| {
        let _ = msg.channel_id.send_message(
            &ctx.http,
            CreateMessage::new().embed(
                default_embed(Some(ctx))
                .title("Donate here!")
                .url("https://www.paypal.me/speckyy")
                .footer(CreateEmbedFooter::new("Thank you all for the support!").icon_url(avatar_url(ctx)))
            )
        ).await;

        Ok(())
    }
}
