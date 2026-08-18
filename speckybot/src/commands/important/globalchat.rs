use serenity::all::CreateMessage;

use crate::{commands::IMPORTANT_CATEGORY, events::global_chat::strings::global_chat_rules};

crate::command!{
    names: ["globalchat","globalchats","gc"],
    category: IMPORTANT_CATEGORY,
    run: |ctx, msg, data| {
        msg.channel_id.send_message(&ctx.http, CreateMessage::new().embed(global_chat_rules())).await?;

        Ok(())
    }
}
