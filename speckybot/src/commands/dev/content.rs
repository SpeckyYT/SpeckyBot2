use serenity::all::CreateMessage;

use crate::util::string::code_block;

crate::command! {
    names: ["content"],
    description: "Shows the content of the parsed command data",
    category: Dev,
    run: |ctx, msg, data| {
        msg.channel_id.send_message(&ctx, CreateMessage::new().content(code_block(format!("{:#?}", data), "rs"))).await?;
        Ok(())
    }
}
