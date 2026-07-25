use serenity::all::{CreateMessage, Permissions};

use crate::util::embed::default_embed;

crate::command! {
    names: ["invite","inv","invit"],
    category: "info",
    run: |ctx, msg, _data| {
        let permissions = Permissions::dm_permissions().bits();

        let id = match ctx.http.application_id() {
            Some(id) => id.get(),
            None => ctx.http.get_current_application_info().await?.id.get(),
        };

        let invite = format!("https://discord.com/api/oauth2/authorize?client_id={id}&scope=bot&permissions={permissions}");

        let _ = msg.channel_id.send_message(
            &ctx.http,
            CreateMessage::new().embed(
                default_embed(Some(ctx))
                .title("Links!")
                .description(format!("[SpeckyBot Discord Server](https://discord.gg/4EecFku)\n[Bot Invite]({invite})\n[Support This Bot](https://www.paypal.me/speckyy)"))
            )
        ).await;

        Ok(())
    }
}
