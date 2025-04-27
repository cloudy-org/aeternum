use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct KeyBinds {
    #[serde(default = "about_box_default")]
    pub about_box: String,
}

impl Default for KeyBinds {
    fn default() -> Self {
        Self {
            about_box: "A".to_string()
        }
    }
}

fn about_box_default() -> String {
    "A".to_string()
}