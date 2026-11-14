use std::sync::LazyLock;

use crate::{commands::COMMANDS_MAP, db::SPECKYBOT_DB};

pub fn load() {
    LazyLock::force(&SPECKYBOT_DB);
    LazyLock::force(&COMMANDS_MAP);
}
