use cirrus_config::v1::config::CConfig;
use serde::{Deserialize, Serialize};

use crate::config::structs::{key_binds::KeyBinds, misc::Misc};

#[derive(Debug, Deserialize, Serialize, Default, Clone)]
pub struct Config {
    #[serde(default = "version_default")]
    pub version: i8,
    #[serde(default)]
    pub keybinds: KeyBinds,
    #[serde(default)]
    pub misc: Misc
}

impl CConfig for Config {}

fn version_default() -> i8 {
    1
}