use serenity::all::{Color, Context, CreateMessage, Message};

use crate::{commands::{self, ParsedCommandData}, env::PREFIX, util::embed::default_embed};

pub async fn on_message(ctx: &Context, msg: &Message) {
    if msg.author.bot { return }
    let Some((_, content)) = msg.content.trim_matches(char::is_whitespace).split_once(&*PREFIX) else { return };
    let content = content.trim_start_matches(char::is_whitespace);
    let mut arguments_iter = content.split(char::is_whitespace).filter(|c| !c.is_empty());
    let Some(command) = arguments_iter.next() else { return };
    let lowercase_command = command.to_lowercase();

    let ctx = ctx.clone();
    let msg = msg.clone();

    match commands::get_run(&lowercase_command) {
        Some(handler) => {
            let cmd_data = ParsedCommandData {
                content: content.to_string(),
                cmd_content: content[command.len()..].trim_start().to_string(),
                args: arguments_iter.map(|s| s.to_string()).collect(),
            };

            tokio::spawn(async move {
                let future = handler(&ctx, &msg, cmd_data);
                if let Err(err) = future.await {
                    let _ = msg.channel_id.send_message(
                        &ctx.http,
                        CreateMessage::new().embed(
                            default_embed(Some(&ctx))
                            .title("Error")
                            .description(format!("{err}"))
                            .color(Color::RED)
                        ),
                    ).await;
                }
            });
        },
        None => {
            match commands::check_category_command(&lowercase_command) {
                Some(category_help) => {
                    tokio::spawn(async move {
                        let _ = msg.channel_id.send_message(&ctx.http, CreateMessage::new().embed(default_embed(Some(&ctx)).description(category_help))).await; 
                    });
                },
                None => {}
            };
        },
    }
}
