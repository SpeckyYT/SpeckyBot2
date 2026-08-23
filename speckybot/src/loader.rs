use std::sync::LazyLock;

use crate::commands::COMMANDS_MAP;

pub fn load() {
    LazyLock::force(&COMMANDS_MAP);
}
