use serenity::all::CreateMessage;
use serenity::async_trait;
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::prelude::*;
use tokio;

use crate::commands::CommandData;
use crate::env::PREFIX;
use crate::util::embed::default_embed;

mod env;
mod commands;
mod util;

struct Bot;

#[async_trait]
impl EventHandler for Bot {
    async fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }
    async fn message(&self, ctx: Context, msg: Message) {
        if msg.author.bot { return }
        let Some((_, content)) = msg.content.trim_matches(char::is_whitespace).split_once(&*PREFIX) else { return };
        let content = content.trim_start_matches(char::is_whitespace);
        let mut arguments_iter = content.split(char::is_whitespace).filter(|c| !c.is_empty());
        let Some(command) = arguments_iter.next() else { return };
        let lowercase_command = command.to_lowercase();

        match commands::get_run(&lowercase_command) {
            Some(handler) => {
                let cmd_data = CommandData {
                    content: content.to_string(),
                    cmd_content: content[command.len()..].trim_start().to_string(),
                    args: arguments_iter.map(|s| s.to_string()).collect(),
                };

                let future = handler(ctx, msg, cmd_data);
                tokio::spawn(async move {
                    if let Err(err) = future.await {
                        println!("command failed: {err:#}");
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
}

#[tokio::main]
async fn main() {
    let _ = env::DOTENV.as_ref();

    // Set gateway intents, which decides what events the bot will be notified about
    let intents = GatewayIntents::GUILD_MESSAGES | GatewayIntents::MESSAGE_CONTENT;

    let mut client = Client::builder(&*env::TOKEN, intents)
        .event_handler(Bot)
        .await
        .expect("Err creating client");

    client.start().await.expect("Client errored");
}
