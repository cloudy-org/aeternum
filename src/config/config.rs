use cirrus_config::v1::config::CConfig;
use serde::{Deserialize, Serialize};

use crate::config::structs::{keybinds::KeyBinds, misc::Misc};

#[derive(Debug, Deserialize, Serialize, Default, Clone)]
pub struct Config {
    pub version: i8,
    pub keybinds: KeyBinds,
    pub misc: Misc
}

impl CConfig for Config {}