use lazy_static::initialize;

use crate::commands::COMMANDS_MAP;

pub fn load() {
    initialize(&COMMANDS_MAP);
}
