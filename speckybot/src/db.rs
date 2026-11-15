use std::path::PathBuf;

use fjall::{Database, Keyspace};

use crate::lazy_lock;

lazy_lock!{
    pub static ref SPECKYBOT_DB: fjall::Result<Database> = Database::builder(PathBuf::from_iter([".", "db", "speckybot"])).open();
    pub static ref BAN_KEYSPACE: anyhow::Result<Keyspace> = SPECKYBOT_DB.as_ref()
        .map_err(|error| anyhow::anyhow!("Failed to open database: {error}"))
        .and_then(|db| db.keyspace("ban", Default::default).map_err(anyhow::Error::from));
}
