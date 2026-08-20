use itertools::Itertools;
use serenity::all::{Context, CreateMessage, Message};

use crate::{commands::{self, COMMANDS_MAP, ParsedCommandData, get_command, is_category_allowed}, env::PREFIX, util::{channels::{COMMAND_ERRORS_CHANNEL, SPECKY_PROJECTS_GUILD}, embed::{default_embed, error_embed}}};

pub async fn on_message(ctx: &Context, msg: &Message) {
    if msg.author.bot { return }
    let Some((_, content)) = msg.content.trim_matches(char::is_whitespace).split_once(&*PREFIX) else { return };
    let content = content.trim_start_matches(char::is_whitespace);
    let mut arguments_iter = content.split(char::is_whitespace).filter(|c| !c.is_empty());
    let Some(command) = arguments_iter.next() else { return };
    let command_length = command.len();
    let lowercase_command = command.to_lowercase();

    let ctx = ctx.clone();
    let msg = msg.clone();

    let metadata_and_run = get_command(&lowercase_command);

    match metadata_and_run {
        Some((metadata, run)) => {
            if let Err(error_text) = is_category_allowed(&ctx, &msg, metadata.category) {
                // TODO: "illegal" feature
                let _ = msg.channel_id.send_message(
                    &ctx.http,
                    CreateMessage::new().embed(
                        error_embed()
                        .description(error_text)
                    )
                ).await;
                return;
            }

            tokio::task::spawn_blocking(move || {
                let ctx = ctx;
                let msg = msg;

                let Some((_, content)) = msg.content.trim_matches(char::is_whitespace).split_once(&*PREFIX) else { return };
                let content = content.trim_start_matches(char::is_whitespace);
                let cmd_data = ParsedCommandData {
                    content,
                    cmd_content: content[command_length..].trim_start(),
                    args: content[command_length..]
                        .split(char::is_whitespace)
                        .filter(|part| !part.is_empty())
                        .collect(),
                };
                
                if let Ok(runtime) = tokio::runtime::Handle::try_current() {
                    runtime.block_on(async {
                        if let Err(err) = run(&ctx, &msg, cmd_data).await {
                            let error_block = format!("```\n{err}\n```");

                            let _ = msg.channel_id.send_message(
                                &ctx.http,
                                CreateMessage::new().embed(
                                    error_embed()
                                    .description(&error_block)
                                ),
                            ).await;

                            if let Some(specky_projects) = ctx.cache.guild(SPECKY_PROJECTS_GUILD)
                                && let Some(command_errors) = specky_projects.channels.get(&COMMAND_ERRORS_CHANNEL) {
                                    let guild = msg.guild(&ctx.cache);
                                    
                                    let message_output = [
                                        Some(format!("Author: {} ({})", msg.author, msg.author.id)),
                                        guild.as_ref()
                                            .and_then(|guild| guild.channels.get(&msg.channel_id))
                                            .map(|channel| format!("Channel: {} ({})", channel, channel.id)),
                                        guild.map(|guild| format!("Guild: {} ({})", guild.name, guild.id)),
                                        COMMANDS_MAP
                                            .get(lowercase_command.as_str())
                                            .map(|(metadata, _)| format!("Command: {lowercase_command} ({})", metadata.names[0])),
                                        Some(error_block),
                                    ]
                                    .into_iter()
                                    .flatten()
                                    .join("\n");

                                    let _ = command_errors.send_message(&ctx.http, CreateMessage::new().content(message_output)).await;
                                }
                        }
                    })
                }
            });
        },
        None => {
            let is_category_allowed = is_category_allowed(&ctx, &msg, &lowercase_command);
            let help_message = commands::check_category_command(&lowercase_command);

            match (is_category_allowed, help_message) {
                (Ok(()), Some(category_help)) => {
                    let _ = msg.channel_id.send_message(&ctx.http, CreateMessage::new().embed(default_embed(Some(&ctx)).description(category_help))).await;
                }
                _ => {} // ignore when not allowed
            }
        },
    }
}
