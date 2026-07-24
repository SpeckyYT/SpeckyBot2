use serenity::all::{Context, CreateMessage, Message};

use crate::{commands::{self, ParsedCommandData, get_command}, env::{PREFIX, is_owner}, util::embed::{default_embed, error_embed}};

const ONWER_ERROR: &str     =  "👮‍♂️ You aren't the bot owner.";
// const BOT_PERM_ERROR: &str  =  "🚫 Bot doesn't have required permissions.";
// const NSFW_ERROR: &str      =  "🔞 This command is only allowed in NSFW channels.";
// const USER_PERM_ERROR: &str =  "🚷 You don't have the required permissions for that command.";
// const SERVER_ERROR: &str    =  "⛔ This command isn't available on this server.";
// const CHANNEL_ERROR: &str   =  "⛔ This command isn't available in this channel.";
// const USER_ERROR: &str      =  "⛔ This command isn't available for you.";
// const OFFICIAL_ERROR: &str  =  "🤖 This is the official SpeckyBot.";

pub async fn on_message(ctx: &Context, msg: &Message) {
    if msg.author.bot { return }
    let Some((_, content)) = msg.content.trim_matches(char::is_whitespace).split_once(&*PREFIX) else { return };
    let content = content.trim_start_matches(char::is_whitespace);
    let mut arguments_iter = content.split(char::is_whitespace).filter(|c| !c.is_empty());
    let Some(command) = arguments_iter.next() else { return };
    let lowercase_command = command.to_lowercase();

    let ctx = ctx.clone();
    let msg = msg.clone();

    let metadata_and_run = get_command(&lowercase_command);

    match metadata_and_run {
        Some((metadata, run)) => {
            if metadata.category == "owner" && !is_owner(msg.author.id.to_string().as_str()) {
                // TODO: "illegal" feature
                let _ = msg.channel_id.send_message(
                    &ctx.http,
                    CreateMessage::new().embed(
                        error_embed()
                        .description(ONWER_ERROR)
                    )
                ).await;
                return;
            }

            let cmd_data = ParsedCommandData {
                content: content.to_string(),
                cmd_content: content[command.len()..].trim_start().to_string(),
                args: arguments_iter.map(|s| s.to_string()).collect(),
            };

            tokio::spawn(async move {
                let future = run(&ctx, &msg, cmd_data);
                if let Err(err) = future.await {
                    let _ = msg.channel_id.send_message(
                        &ctx.http,
                        CreateMessage::new().embed(
                            error_embed()
                            .description(format!("{err}"))
                        ),
                    ).await;
                }
            });
        },
        None => {
            if let Some(category_help) = commands::check_category_command(&lowercase_command) {
                tokio::spawn(async move {
                    let _ = msg.channel_id.send_message(&ctx.http, CreateMessage::new().embed(default_embed(Some(&ctx)).description(category_help))).await; 
                });
            };
        },
    }
}
