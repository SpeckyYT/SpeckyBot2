use rand::random_range;
use serenity::all::CreateMessage;

crate::command! {
    names: ["bubblewrap","bw"],
    description: "Gives you a virtual bubblewrap!",
    category: Games,
    run: |ctx, msg, _data| {
        let bw = ("||pop||".repeat(random_range(7..=10)) + "\n").repeat(random_range(7..=10));
        Ok(msg.channel_id.send_message(&ctx.http, CreateMessage::new().content(bw.trim())).await.map(|_| ())?)
    }
}
