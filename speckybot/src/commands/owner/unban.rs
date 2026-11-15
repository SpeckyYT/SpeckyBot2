use serenity::all::CreateMessage;

use crate::util::embed::success_embed;

crate::command! {
    names: ["unban"],
    category: Owner,
    db: [BAN_KEYSPACE],
    run: |ctx, msg, data| (db, bans) {
        let Ok(id): Result<u64, _> = data.cmd_content.parse() else {
            return Err(anyhow::anyhow!("Input is not a valid Discord ID"));
        };
        let id_string = id.to_string();
        if let Err(err) = bans.remove(&id_string) {
            return Err(anyhow::anyhow!("Failed to remove ban into keyspace: {err}"))
        }
        let response = CreateMessage::new().add_embed(success_embed().description(format!("Successfully unbanned user {id_string}")));
        msg.channel_id.send_message(&ctx, response).await?;
        Ok(())
    }
}
