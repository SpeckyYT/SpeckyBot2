use serenity::all::EditMessage;

use crate::commands::IMPORTANT_CATEGORY;

crate::command! {
    names: ["ping", "pong", "pin", "pon"],
    category: IMPORTANT_CATEGORY,
    run: |ctx, msg, _data| {
        let mut bot_msg = msg.channel_id.say(&ctx.http, "pong").await?;
        let time = bot_msg.timestamp.to_utc() - msg.timestamp.to_utc();
        let ms = time.as_seconds_f32() * 1000.0;
        bot_msg.edit(&ctx.http, EditMessage::new().content(format!("pong ({ms:.0?}ms)"))).await?;
        Ok(())
    },
}
