use std::path::PathBuf;
use dotenv::dotenv;
use lazy_static::lazy_static;

lazy_static! {
    pub static ref DOTENV: Option<PathBuf> = dotenv().ok();
    pub static ref TOKEN: String = std::env::var("DISCORD_TOKEN").expect("discord bot token [.env]");
    pub static ref PREFIX: String = std::env::var("PREFIX").expect("prefix string [.env]");
    pub static ref COLOR: String = std::env::var("COLOR").unwrap_or("FF00AA".to_string());
}
