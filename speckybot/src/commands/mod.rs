use itertools::Itertools;
use serenity::all::{Context, Message};
use std::pin::Pin;

use crate::{env::{PREFIX, is_owner}, output_file, util::{channels::is_nsfw_channel, string::uppercase_first_char}};

pub type RunFuture<'a> = Pin<Box<dyn std::future::Future<Output = anyhow::Result<()>> + Send + 'a>>;
pub type RunFunction = Box<
    dyn for<'a> Fn(
        &'a serenity::client::Context,
        &'a serenity::model::channel::Message,
        ParsedCommandData,
    ) -> RunFuture<'a>
        + Send
        + Sync,
>;

pub const DEFAULT_CATEGORY: &str = "uncategorized";
pub const DEFUALT_DESCRIPTION: &str = "No description provided";
pub const DEFUALT_USAGE: &str = "No usage provided";

pub const IMPORTANT_CATEGORY: &str = "important";
pub const MATH_CATEGORY: &str = "math";
pub const OWNER_CATEGORY: &str = "owner";
pub const GAMES_CATEGORY: &str = "games";
pub const NSFW_CATEGORY: &str = "nsfw";

pub const OWNER_ERROR: &str     =  "👮‍♂️ You aren't the bot owner.";
// const BOT_PERM_ERROR: &str  =  "🚫 Bot doesn't have required permissions.";
pub const NSFW_ERROR: &str      =  "🔞 This command is only allowed in NSFW channels.";
// const USER_PERM_ERROR: &str =  "🚷 You don't have the required permissions for that command.";
// const SERVER_ERROR: &str    =  "⛔ This command isn't available on this server.";
// const CHANNEL_ERROR: &str   =  "⛔ This command isn't available in this channel.";
// const USER_ERROR: &str      =  "⛔ This command isn't available for you.";
// const OFFICIAL_ERROR: &str  =  "🤖 This is the official SpeckyBot.";

#[derive(Debug, Clone, Copy)]
pub struct CommandMetadata {
    pub names: &'static [&'static str],
    pub description: &'static str,
    pub usage: &'static str,
    pub category: &'static str,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
pub struct ParsedCommandData {
    /// # content
    /// ```
    /// "sb!help help"
    ///  ^^^^^^^^^^^^
    /// "sb!help help"
    /// ```
    /// Contains exactly the same content as `msg.content`
    pub content: String,
    /// # cmd_content
    /// ```
    /// "sb!help help"
    ///          ^^^^
    ///         "help"
    /// ```
    /// Filters out the prefix and command and trims start
    pub cmd_content: String,
    /// # args
    /// ```
    /// "sb!help arg1 arg2 arg3 arg4 arg5"
    ///          ^^^^ ^^^^ ^^^^ ^^^^ ^^^^
    /// ["arg1","arg2","arg3","arg4","arg5"]
    /// ```
    pub args: Vec<String>,
}

#[macro_export]
macro_rules! command {
    (
        names: $names:tt,
        $(description: $desc:literal,)?
        $(usage: $usage:literal,)?
        $(category: $cat:expr,)?
        run: |$ctx:ident, $msg:ident, $content:ident| $body:block
        $(,)*
    ) => {
        pub const METADATA: $crate::commands::CommandMetadata = $crate::commands::CommandMetadata {
            names: command!(@names: $names),
            description: [$($desc,)? $crate::commands::DEFUALT_DESCRIPTION][0],
            usage: [$($usage,)? $crate::commands::DEFUALT_USAGE][0],
            category: [$($cat,)? $crate::commands::DEFAULT_CATEGORY][0],
        };
        
        pub async fn run(
            #[allow(unused)]
            $ctx: &serenity::client::Context,
            #[allow(unused)]
            $msg: &serenity::model::channel::Message,
            #[allow(unused)]
            $content: $crate::commands::ParsedCommandData,
        ) -> anyhow::Result<()> $body

        // // Poise slash command version
        // #[poise::command(slash_command)]
        // pub async fn slash_command_impl(
        //     $ctx: poise::Context<'_, (), serenity::prelude::SerenityError>,
        // ) -> Result<(), poise::serenity_prelude::Error> {
        //     // Adapter to call your existing run function
        //     let msg = $ctx.msg().cloned().unwrap_or_else(|| {
        //         // Create a minimal message for slash commands if needed
        //         serenity::model::channel::Message::default()
        //     });
            
        //     let _ = run(&$ctx.serenity_context(), &msg, Default::default()).await;
        //     Ok(())
        // }
    };
    (@names: $name:literal) => { &[$name] };
    (@names: [$($names:literal),+ $(,)?]) => { &[$($names),+] };
}

#[allow(unused)]
macro_rules! commands {
    ($($name:ident $(: $str:literal)?),* $(,)?) => {
        $(
            $(#[path = $str])?
            mod $name;
        )*

        use ahash::AHashMap;
        use std::collections::BTreeMap;

        lazy_static::lazy_static! {
            pub static ref COMMANDS_ARRAY: &'static [CommandMetadata] = &[ $($name::METADATA),* ];
            pub static ref COMMANDS_MAP: AHashMap<&'static str, (CommandMetadata, RunFunction)> = {
                let mut map = AHashMap::new();
                $(
                    for &command_name in $name::METADATA.names {
                        let metadata = $name::METADATA;
                        let run: RunFunction = Box::new(|ctx, msg, cont| Box::pin($name::run(ctx, msg, cont)));
                        if map.insert(command_name, (metadata, run)).is_some() {
                            panic!("Duplicate command name `{command_name}`");
                        }
                    }
                )*
                map
            };
            pub static ref CATEGORIES: BTreeMap<&'static str, Vec<&'static CommandMetadata>> = {
                let mut commands_map: BTreeMap<&'static str, Vec<&'static CommandMetadata>> = BTreeMap::new();
                COMMANDS_ARRAY.into_iter().for_each(|c| commands_map.entry(c.category).or_insert(Vec::with_capacity(COMMANDS_ARRAY.len())).push(c));
                commands_map
            };
        }
    };
}

#[inline]
pub fn get_command(command_name: &str) -> Option<&'static (CommandMetadata, RunFunction)> {
    COMMANDS_MAP.get(command_name)
}

#[inline]
pub fn is_category_allowed(ctx: &Context, msg: &Message, category: &str) -> Result<(), &'static str> {
    match category {
        OWNER_CATEGORY if !is_owner(msg.author.id.to_string()) => Err(OWNER_ERROR),
        NSFW_CATEGORY if !is_nsfw_channel(ctx, msg) => Err(NSFW_ERROR),
        _ => Ok(()),
    }
}

#[inline]
pub fn check_category_command(category: &str) -> Option<String> {
    CATEGORIES.get(category)
        .map(|metadatas| {
            #[allow(unstable_name_collisions)]
            let commands: String = metadatas.iter().map(|c| format!("+ {}", c.names[0])).intersperse("\n".to_string()).collect();
            format!("The bot prefix is: **{}**\n\n> **{}**\n```diff\n{commands}\n```", *PREFIX, uppercase_first_char(category))
        })
}

include!(concat!(env!("OUT_DIR"), "/", output_file!()));
