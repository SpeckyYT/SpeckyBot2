use std::path::PathBuf;

use fjall::Database;

use crate::lazy_lock;

lazy_lock!{
    pub static ref SPECKYBOT_DB: Database = Database::builder(PathBuf::from_iter([".", "db", "speckybot"])).open().unwrap();
}
