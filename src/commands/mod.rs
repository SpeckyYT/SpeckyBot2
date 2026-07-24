use ahash::AHashMap;
use itertools::Itertools;
use std::{collections::BTreeMap, pin::Pin};

use crate::{env::PREFIX, util::string::uppercase_first_char};

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

#[derive(Debug, Clone, Copy)]
pub struct CommandMetadata {
    pub names: &'static [&'static str],
    pub description: &'static str,
    pub usage: &'static str,
    pub category: &'static str,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ParsedCommandData {
    pub content: String,
    pub cmd_content: String,
    pub args: Vec<String>,
}

#[macro_export]
macro_rules! command {
    (
        names: $names:tt,
        $(description: $desc:literal,)?
        $(usage: $usage:literal,)?
        $(category: $cat:literal,)?
        run: |$ctx:ident, $msg:ident, $content:ident| $body:block
        $(,)*
    ) => {
        pub const METADATA: $crate::commands::CommandMetadata = $crate::commands::CommandMetadata {
            names: command!(@names: $names),
            description: [$($desc,)? $crate::commands::DEFUALT_DESCRIPTION][0],
            usage: [$($usage,)? $crate::commands::DEFUALT_USAGE][0],
            category: [$($cat,)? $crate::commands::DEFAULT_CATEGORY][0],
        };
        pub async fn run($ctx: &serenity::client::Context, $msg: &serenity::model::channel::Message, $content: $crate::ParsedCommandData) -> anyhow::Result<()> $body
    };
    (@names: $name:literal) => { &[$name] };
    (@names: [$($names:literal),+ $(,)?]) => { &[$($names),+] };
}

macro_rules! commands {
    ($($name:ident $(: $str:literal)?),* $(,)?) => {
        $(
            $(#[path = $str])?
            mod $name;
        )*

        lazy_static::lazy_static! {
            static ref COMMANDS_ARRAY: &'static [CommandMetadata] = &[ $($name::METADATA),* ];
            static ref COMMANDS_MAP: AHashMap<&'static str, (CommandMetadata, RunFunction)> = {
                let mut map = AHashMap::new();
                $(
                    for &command_name in $name::METADATA.names {
                        let metadata = $name::METADATA;
                        let run: RunFunction = Box::new(|ctx, msg, cont| Box::pin($name::run(ctx, msg, cont)));
                        map.insert(command_name, (metadata, run));
                    }
                )*
                map
            };
            static ref CATEGORIES: BTreeMap<&'static str, Vec<&'static CommandMetadata>> = {
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

pub fn check_category_command(category: &str) -> Option<String> {
    CATEGORIES.get(category)
        .map(|metadatas| {
            #[allow(unstable_name_collisions)]
            let commands: String = metadatas.iter().map(|c| format!("+ {}", c.names[0])).intersperse("\n".to_string()).collect();
            format!("The bot prefix is: **{}**\n\n> **{}**\n```diff\n{commands}\n```", &*PREFIX, uppercase_first_char(category))
        })
}

commands![
    help,
    ping,
    update: "owner/update.rs",
];
