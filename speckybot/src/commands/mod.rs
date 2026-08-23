use enum_map::Enum;
use itertools::Itertools;
use serenity::all::{Context, Message};
use std::pin::Pin;

use crate::{env::{PREFIX, is_owner}, output_file, util::channels::is_nsfw_channel};

pub type RunFuture<'a> = Pin<Box<dyn std::future::Future<Output = anyhow::Result<()>> + Send + 'a>>;
pub type RunFunction = Box<
    dyn for<'a> Fn(
        &'a serenity::client::Context,
        &'a serenity::model::channel::Message,
        ParsedCommandData<'a>,
    ) -> RunFuture<'a>
        + Send
        + Sync,
>;
pub const DEFUALT_DESCRIPTION: &str = "No description provided";
pub const DEFUALT_USAGE: &str = "No usage provided";

#[derive(Debug, Clone, Copy, Default, Enum)]
pub enum Category {
    Important,
    Games,
    Math,
    Nsfw,
    Owner,
    #[default]
    Uncategorized,
}

impl Into<&'static str> for Category {
    fn into(self) -> &'static str {
        match self {
            Category::Important => "important",
            Category::Games => "games",
            Category::Math => "math",
            Category::Nsfw => "nsfw",
            Category::Owner => "owner",
            Category::Uncategorized => "uncategorized",
        }
    }
}

impl Category {
    pub fn uppercase_first_char(self) -> &'static str {
        match self {
            Category::Important => "Important",
            Category::Games => "Games",
            Category::Math => "Math",
            Category::Nsfw => "NSFW",
            Category::Owner => "Owner",
            Category::Uncategorized => "Uncategorized",
        }
    }
    pub fn from_str(string: &str) -> Option<Self> {
        Some(match string {
            "important" => Category::Important,
            "games" => Category::Games,
            "math" => Category::Math,
            "nsfw" => Category::Nsfw,
            "owner" => Category::Owner,
            "uncategorized" => Category::Uncategorized,
            _ => return None,
        })
    }
}

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
    pub category: Category,
}

#[derive(Debug, Clone, Default)]
pub struct ParsedCommandData<'a> {
    /// # content
    /// ```
    /// "sb!help help"
    ///  ^^^^^^^^^^^^
    /// "sb!help help"
    /// ```
    /// Contains exactly the same content as `msg.content`
    pub content: &'a str,
    /// # cmd_content
    /// ```
    /// "sb!help help"
    ///          ^^^^
    ///         "help"
    /// ```
    /// Filters out the prefix and command and trims start
    pub cmd_content: &'a str,
    /// # args
    /// ```
    /// "sb!help arg1 arg2 arg3 arg4 arg5"
    ///          ^^^^ ^^^^ ^^^^ ^^^^ ^^^^
    /// ["arg1","arg2","arg3","arg4","arg5"]
    /// ```
    pub args: Vec<&'a str>,
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
        #[allow(unused_imports)]
        use $crate::commands::Category::*;

        pub const METADATA: $crate::commands::CommandMetadata = $crate::commands::CommandMetadata {
            names: command!(@names: $names),
            description: [$($desc,)? $crate::commands::DEFUALT_DESCRIPTION][0],
            usage: [$($usage,)? $crate::commands::DEFUALT_USAGE][0],
            category: [$($cat,)? $crate::commands::Category::Uncategorized][0],
        };
        
        pub async fn run<'a>(
            #[allow(unused)]
            $ctx: &'a serenity::client::Context,
            #[allow(unused)]
            $msg: &'a serenity::model::channel::Message,
            #[allow(unused)]
            $content: $crate::commands::ParsedCommandData<'a>,
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
        use enum_map::EnumMap;

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
            pub static ref CATEGORIES: EnumMap<Category, Vec<&'static CommandMetadata>> = {
                let mut commands_map: EnumMap<Category, Vec<&'static CommandMetadata>> = EnumMap::from_fn(|_| vec![]);
                COMMANDS_ARRAY.into_iter().for_each(|c| commands_map[c.category].push(c));
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
pub fn is_category_allowed(ctx: &Context, msg: &Message, category: Category) -> Result<(), &'static str> {
    match category {
        Category::Owner if !is_owner(msg.author.id.to_string()) => Err(OWNER_ERROR),
        Category::Nsfw if !is_nsfw_channel(ctx, msg) => Err(NSFW_ERROR),
        _ => Ok(()),
    }
}

#[inline]
pub fn category_help_message(category: Category) -> String {
    #[allow(unstable_name_collisions)]
    let commands: String = CATEGORIES[category].iter().map(|c| format!("+ {}", c.names[0])).intersperse("\n".to_string()).collect();
    format!("The bot prefix is: **{}**\n\n> **{}**\n```diff\n{commands}\n```", *PREFIX, category.uppercase_first_char())
}

include!(concat!(env!("OUT_DIR"), "/", output_file!()));
