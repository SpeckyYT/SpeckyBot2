use serenity::all::CreateMessage;

use crate::util::embed::success_embed;

crate::command! {
    names: ["ban"],
    category: Owner,
    db: [BAN_KEYSPACE] (db, bans),
    run: |ctx, msg, data| {
        let Ok(id): Result<u64, _> = data.cmd_content.parse() else {
            return Err(anyhow::anyhow!("Input is not a valid Discord ID"));
        };
        let id_string = id.to_string();
        if let Err(err) = bans.insert(&id_string, chrono::Local::now().to_rfc3339()) {
            return Err(anyhow::anyhow!("Failed to insert ban into keyspace: {err}"))
        }
        let response = CreateMessage::new().add_embed(success_embed().description(format!("Successfully banned user {id_string}")));
        msg.channel_id.send_message(&ctx, response).await?;
        Ok(())
    }
}
