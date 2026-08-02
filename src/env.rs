use std::path::PathBuf;
use dotenv::dotenv;
use lazy_static::lazy_static;

lazy_static! {
    pub static ref DOTENV: Option<PathBuf> = dotenv().ok();
    pub static ref TOKEN: String = std::env::var("DISCORD_TOKEN").expect("discord bot token [.env]");
    pub static ref PREFIX: String = std::env::var("PREFIX").expect("prefix string [.env]");
    pub static ref COLOR: String = std::env::var("COLOR").unwrap_or("FF00AA".to_string());
    pub static ref OWNERS_ENV: String = std::env::var("OWNERS").unwrap_or_default();
    pub static ref OWNERS: Vec<&'static str> = OWNERS_ENV.split(|c: char| !c.is_ascii_digit()).filter(|id| !id.is_empty()).collect();
    pub static ref GLOBAL_CHAT: bool = std::env::var("GLOBAL_CHAT").unwrap_or_default().contains("true");
}

pub fn is_owner(id: &str) -> bool {
    OWNERS.contains(&id)
}
