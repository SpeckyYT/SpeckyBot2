use serenity::all::CreateEmbedFooter;
use serenity::builder::CreateEmbed;

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

pub fn global_chat_rules(global_chats_count: usize) -> CreateEmbed {
    let user_rules = [
        "Be sure to follow the following rules!",
        "No NSFW/NSFL or similars. Don't send anything related to +18/illegal/disturbing/unsettling content.",
        "Don't spam. This includes sending earrape videos, sending messages with little to no content, sending huge messages, sending too many attachments and disturbing ongoing discussions.",
        "Don't use bot commands (of any bot). Use a bot-channel instead.",
        "Don't advertise. Do that in other channels where it's permitted.",
        "Be as nice as possible to everyone.",
        "Don't try to bypass any of the automatic filters/limitations.",
    ];
    
    let admin_rules = [
        "The next rules/informations are specifically for server admins/moderators.",
        "It's highly recommended to create a new/separate channel for the global-chat.",
        "The global-chat channel can't be tagged as NSFW.",
        "Every server should moderate it's own server of the global-chat.",
        "If anyone breaks one of the rules above, the moderation team of that server should delete the message.",
        "If big part of a server doesn't follow the rules, the server may get banned from using SpeckyBot.",
        // format!("Your server will have to have at least {} members.", min_members).as_str(), // TODO
        "SpeckyBot requires the following permissions: read/send messages/files/embeds and manage messages",
    ];
    
    let reactions = [
        "On some messages, you may get a reaction right after sending (the message gets ignored).".to_string(),
        format!("{}: Don't send two or more messages in a row", EMOTES.not_twice),
        format!("{}: Your message is too big", EMOTES.too_long),
        format!("{}: Your message contains external emotes", EMOTES.no_external),
    ];
    
    let notes = [
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
    
    let tldr = [
        "too long; didn't read.",
        "Don't be a dumbass.",
        "Don't share private/personal data.",
        "Read the entire page you lazy ass.",
        "Have fun.",
    ];
    
    let format_rules = |rules: &[&str]| {
        rules
            .iter()
            .enumerate()
            .map(|(i, rule)| {
                if i == 0 {
                    format!("# {}", rule)
                } else {
                    format!("{}. {}", i, rule)
                }
            })
            .collect::<Vec<_>>()
            .join("\n")
    };
    
    let format_notes = |notes: &[&str]| {
        notes
            .iter()
            .enumerate()
            .map(|(i, note)| {
                if i == 0 {
                    format!("+ {}", note)
                } else {
                    format!("- {}", note)
                }
            })
            .collect::<Vec<_>>()
            .join("\n")
    };
    
    let format_tldr = |tldr: &[&str]| {
        tldr
            .iter()
            .enumerate()
            .map(|(i, item)| {
                if i == 0 {
                    format!("# {}", item)
                } else {
                    format!("[{}]: {}", i, item)
                }
            })
            .collect::<Vec<_>>()
            .join("\n")
    };
    
    CreateEmbed::default()
        .title("Global Chat!")
        .description(format!(
            "```fix\n{}\n```\n```c\n{}\n```",
            "The Global Chat is a cross-server channel which allows you to make new friends, ask questions, talk about general stuff and much more!",
            "By including \"[GLOBAL]\" into a channel's topic, the channel will turn into a global-chat!"
        ))
        .field("User Rules", format!("```md\n{}\n```", format_rules(&user_rules)), false)
        .field("Mods/Admins Rules/Informations", format!("```md\n{}\n```", format_rules(&admin_rules)), false)
        .field("Reactions", reactions.join("\n"), false)
        .field("Notes", format!("```diff\n{}\n```", format_notes(&notes)), false)
        .field("TL;DR", format!("```md\n{}\n```", format_tldr(&tldr)), false)
        .footer(CreateEmbedFooter::new(format!("{} Global Chats Connected", global_chats_count)))
}
