use std::borrow::Cow;

use ascii_table::AsciiTable;
use serenity::all::{CreateEmbedFooter, CreateMessage};

use crate::{commands::{self, CATEGORIES, COMMANDS_ARRAY, CommandMetadata, IMPORTANT_CATEGORY, NSFW_CATEGORY, OWNER_CATEGORY}, env::{PREFIX, is_owner}, events::commands::{NSFW_ERROR, ONWER_ERROR}, holy_cow, util::{bot_user::bot_avatar_url, channels::is_nsfw_channel, embed::default_embed}};

holy_cow![
    DID_U_KNOW
    // "you can use the `${bot.config.prefix}usersettings` command to personalize your experience!",
    // "you can send a message that contains `:EMB:` to turn your message into an embed!",
    // "you can include `--emb` in the `${bot.config.prefix}say` command to turn the text into an embed!",
    // "you can type in a channel topic `Next number: 1` to turn it into a counting-up channel!",
    // "in any text channel, you can include `[ALTERNATE]` in the channel topic, so all users have to alternate!",
    // "in any text channel, you can include `[GLOBAL]` in the channel topic, so you can chat with all users of the world!",
    // "in any text channel, you can include `[ONE-WORD]` in the channel topic, so all users can only type one word per message!",
    // "in any text channel, you can include `[NO-MEDIA]` in the channel topic, so nobody can share links/images in the channel!",
    // "in any text channel, you can include `[NO-NSFW]` in the channel topic, so every NSFW command is not executable!",
    format!("commands usually have aliases? Just execute the command `{}help <command>` to check them!", &*PREFIX),
    "most of the people don't read the helpful tricks that are written here?",
    format!("I am a bot and I'm forced for my entire life to do this 😭 Please send help, `{}donation`", &*PREFIX),
];
crate::command! {
    names: ["help", "h", "halp", "hel","hwlp","hewlp","cmd","cmds","command","commands","info","informations","information","?"],
    category: IMPORTANT_CATEGORY,
    run: |ctx, msg, data| {
        let embed = default_embed(Some(ctx));
        let embed = match data.args.first().map(|cmd| commands::get_command(&cmd.to_lowercase())) {
            // OWNER COMMAND AND IS NOT OWNER
            Some(Some((cmd,_))) if cmd.category == OWNER_CATEGORY && !is_owner(msg.author.id.to_string()) => embed.description(ONWER_ERROR),
            // NSFW COMMAND AND IS NOT IN NSFW CHANNEL
            Some(Some((cmd,_))) if cmd.category == NSFW_CATEGORY && !is_nsfw_channel(ctx, msg) => embed.description(NSFW_ERROR),
            
            // COMMAND FOUND
            Some(Some((CommandMetadata { names, description, category, usage, .. }, _))) => {
                let mut command_info = format!(
                    "The bot's prefix is: `{}`\n\n**Command:** {}\n**Category:** {category}\n**Description:** {description}\n **Usage:** {usage}\n",
                    *PREFIX,
                    names[0],
                );
                if names.len() > 1 {
                    command_info.push_str(&format!("**Aliases:** {}", names.join(", ")));
                }
                embed.description(command_info)
            }
            // COMMAND NOT FOUND
            Some(None) => {
                embed.title("Invalid Command")
                    .description(format!("Do `{}help` for the list of commands", *PREFIX))
            }
            // GENERAL HELP MESSAGE
            None => {
                let bot_user = ctx.cache.current_user();

                let filtered_categories = CATEGORIES.iter()
                .filter(|&c| match *c.0 {
                    OWNER_CATEGORY => is_owner(msg.author.id.to_string()),
                    _ => true
                });

                let mut table = AsciiTable::new();
                table.column(0)
                    .set_header("category")
                    .set_align(ascii_table::Align::Left);
                table.column(1)
                    .set_header("commands")
                    .set_align(ascii_table::Align::Right);
                let mut table_chars = Vec::new();
                let categories_table: Vec<_> = filtered_categories.map(|(&x,y)| [Cow::Borrowed(x), Cow::Owned(y.len().to_string())]).collect();
                table.writeln(&mut table_chars, &categories_table)?;
                let table_string = format!("```\n{}\n```", String::from_utf8_lossy(&table_chars));

                embed
                    .description(format!("These are the available commands for {}\nThe bot prefix is: **{}**\n{table_string}", bot_user.name, *PREFIX))
                    .field("Instructions", format!("Simple! Just type `{}<category>` ||(without <> obviously)|| to get the available commands of the categories!", *PREFIX), false)
                    .field("Did you know that", &*DID_U_KNOW[rand::random_range(0..DID_U_KNOW.len())], false)
                    .footer(CreateEmbedFooter::new(format!("Based on SpeckyBot2 | Total Commands: {}", COMMANDS_ARRAY.len())).icon_url(bot_avatar_url(ctx)))
            }
        };

        msg.channel_id.send_message(&ctx.http, CreateMessage::new().add_embed(embed)).await?;

        Ok(())
    },
}
