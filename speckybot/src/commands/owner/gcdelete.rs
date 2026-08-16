use anyhow::anyhow;
use serenity::all::{Color, CreateMessage, MessageId};

use crate::{commands::OWNER_CATEGORY, events::global_chat::gc_messages, util::embed::default_embed};

crate::command! {
    names: "update",
    category: OWNER_CATEGORY,
    run: |ctx, msg, data| {
        let gc_messages = gc_messages();

        let Ok(m_id) = data.content.parse().map(|id| MessageId::new(id)) else {
            return Err(anyhow!("Input message ID isn't valid"));
        };
            
        let Some(message) = gc_messages.get(&m_id) else {
            return Err(anyhow!("Message ID not found"));
        };

        match message.delete_message_family(ctx).await {
            Some(messages) => {
                let total_amount = messages.len();
                let success_amount = messages.iter().flatten().count();
                let content = format!("Messages deleted `{success_amount}` (ouf of `{total_amount}`)");

                msg.channel_id.send_message(&ctx.http, CreateMessage::new().embed(default_embed(Some(ctx)).color(Color::DARK_GREEN).description(content))).await?;
            }
            None => return Err(anyhow!("Failed to delete messages")),
        }

        Ok(())
    }
}
