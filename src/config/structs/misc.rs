use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Misc {
    #[serde(default = "enable_default")] 
    pub enable_custom_folder: bool
}


impl Default for Misc {
    fn default() -> Self {
        Self {
            enable_custom_folder: true
        }
    }
}

fn enable_default() -> bool {
    true
}