use rand::random_range;
use serenity::all::CreateMessage;

use crate::commands::GAMES_CATEGORY;

crate::command! {
    names: ["bubblewrap","bw"],
    category: GAMES_CATEGORY,
    run: |ctx, msg, _data| {
        let bw = ("||pop||".repeat(random_range(7..=10)) + "\n").repeat(random_range(7..=10));
        Ok(msg.channel_id.send_message(&ctx.http, CreateMessage::new().content(bw.trim())).await.map(|_| ())?)
    }
}
