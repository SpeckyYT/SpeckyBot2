use serenity::all::CreateMessage;

crate::command! {
    names: ["bans"],
    description: "Shows the list of banned users",
    category: Owner,
    db: [BAN_KEYSPACE] (db, bans),
    run: |ctx, msg, data| {
        let bans_string = bans
            .iter()
            .flat_map(|guard| guard.into_inner())
            .fold(String::new(), |mut string, (id, time)| {
                string.push_str(&String::from_utf8_lossy(id.as_slice()));
                string.push_str(": ");
                string.push_str(&String::from_utf8_lossy(time.as_slice()));
                string.push('\n');
                string
            });
        msg.channel_id.send_message(&ctx, CreateMessage::new().content(bans_string.trim())).await?;
        Ok(())
    }
}
