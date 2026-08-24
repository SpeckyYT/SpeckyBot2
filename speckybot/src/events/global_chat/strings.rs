use itertools::Itertools;
use serenity::all::CreateEmbedFooter;
use serenity::builder::CreateEmbed;

use crate::events::global_chat::gc_channels;
use crate::{holy_cow, lazy_lock};
use crate::util::string::code_block;

pub struct Emotes {
    pub not_twice: &'static str,
    pub too_long: &'static str,
    pub no_external: &'static str,
}


const EMOTES: Emotes = Emotes {
    not_twice: "2️⃣",
    no_external: "🌐",
    too_long: "📏",
};

const USER_RULES: &[&str] = &[
    "Be sure to follow the following rules!",
    "No NSFW/NSFL or similars. Don't send anything related to +18/illegal/disturbing/unsettling content.",
    "Don't spam. This includes sending earrape videos, sending messages with little to no content, sending huge messages, sending too many attachments and disturbing ongoing discussions.",
    "Don't use bot commands (of any bot). Use a bot-channel instead.",
    "Don't advertise. Do that in other channels where it's permitted.",
    "Be as nice as possible to everyone.",
    "Don't try to bypass any of the automatic filters/limitations.",
];

const ADMIN_RULES: &[&str] = &[
    "The next rules/informations are specifically for server admins/moderators.",
    "It's highly recommended to create a new/separate channel for the global-chat.",
    "The global-chat channel can't be tagged as NSFW.",
    "Every server should moderate it's own server of the global-chat.",
    "If anyone breaks one of the rules above, the moderation team of that server should delete the message.",
    "If big part of a server doesn't follow the rules, the server may get banned from using SpeckyBot.",
    // format!("Your server will have to have at least {} members.", min_members).as_str(), // TODO
    "SpeckyBot requires the following permissions: read/send messages/files/embeds and manage messages",
];

holy_cow![
    REACTIONS
    "On some messages, you may get a reaction right after sending (the message gets ignored).",
    format!("{}: Don't send two or more messages in a row", EMOTES.not_twice),
    format!("{}: Your message is too big", EMOTES.too_long),
    format!("{}: Your message contains external emotes", EMOTES.no_external),
];

const NOTES: &[&str] = &[
    "Note:",
    "People may be young, have epilepsy, have heart problems or other psychophysical problems, so be sure to act accordingly.",
    "Every channel connected to the global-chat can read your messages.",
    "Everyone will see your username (and icon)",
    "Everyone will see the name of the server you're writing in (and icon)",
    "Editing and deleting messages is possible.",
    "Sending images is allowed (if they're not against the rules)",
    "Every message in the global-chat will get processed.",
    "Rules may be subject to changes at any time",
];

const TLDR: &[&str] = &[
    "too long; didn't read.",
    "Don't be a dumbass.",
    "Don't share private/personal data.",
    "Read the entire page you lazy ass.",
    "Have fun.",
];

lazy_lock!{
    pub static ref EXPLANATION: String = {
        let explaination = code_block("The Global Chat is a cross-server channel which allows you to make new friends, ask questions, talk about general stuff and much more!", "fix");
        let setup_explain = code_block("By including \"[GLOBAL]\" into a channel's topic, the channel will turn into a global-chat!", "c");
        format!("{explaination}\n{setup_explain}")

    };
    pub static ref USER_RULES_STRING: String = code_block(format_lines(USER_RULES.iter(), |(i, rule)| {
        if i == 0 {
            format!("# {}", rule)
        } else {
            format!("{}. {}", i, rule)
        }
    }), "md");
    pub static ref ADMIN_RULES_STRING: String = code_block(format_lines(ADMIN_RULES.iter(), |(i, rule)| {
        if i == 0 {
            format!("# {}", rule)
        } else {
            format!("{}. {}", i, rule)
        }
    }), "md");
    pub static ref REACTIONS_STRING: String = REACTIONS.join("\n");
    pub static ref NOTES_STRING: String = code_block(format_lines(NOTES.iter(), |(i, note)| {
        if i == 0 {
            format!("+ {}", note)
        } else {
            format!("- {}", note)
        }
    }), "diff");
    pub static ref TLDR_STRING: String = code_block(format_lines(TLDR.iter(), |(i, item)| {
        if i == 0 {
            format!("# {}", item)
        } else {
            format!("[{}]: {}", i, item)
        }
    }),"md");
}

#[inline]
fn format_lines<I, S, F>(input: I, map: F) -> String
where
    I: Iterator<Item = S>,
    S: AsRef<str>,
    F: FnMut((usize, S)) -> String,
{
    #[allow(unstable_name_collisions)]
    input
        .enumerate()
        .map(map)
        .intersperse("\n".to_string())
        .collect()
}

pub fn global_chat_rules() -> CreateEmbed {
    CreateEmbed::default()
        .title("Global Chat!")
        .description(&*EXPLANATION)
        .field("User Rules", &*USER_RULES_STRING, false)
        .field("Mods/Admins Rules/Informations", &*ADMIN_RULES_STRING, false)
        .field("Reactions", &*REACTIONS_STRING, false)
        .field("Notes", &*NOTES_STRING, false)
        .field("TL;DR", &*TLDR_STRING, false)
        .footer(CreateEmbedFooter::new(format!("{} Global Chats Connected", gc_channels().len())))
}
