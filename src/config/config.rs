use log::debug;
use serde::{Deserialize, Serialize};

use std::{error::Error, fs};
use crate::config::structs::{keybinds::KeyBinds, misc::Misc};

#[derive(Debug, Deserialize, Serialize, Default, Clone)]
pub struct Config {
    pub version: i8,
    pub keybinds: KeyBinds,

    pub misc: Misc
}

impl Config {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        debug!("Finding operating system's configuration local directory...");
        let local_config_dir = match dirs::config_local_dir() {
            Some(dir) => dir,
            None => {
                return Err("No config path was found for your OS!?".into());
            }
        };

        let aeternum_config_dir_path = local_config_dir.join("cloudy").join("aeternum");

        if !aeternum_config_dir_path.exists() {
            debug!("Creating config directory for aeternum...");
            if let Err(err) = fs::create_dir_all(&aeternum_config_dir_path) {
                return Err(
                    format!("Unable to create config path: {}", err).into()
                );
            };

            debug!("Config directory created!");
        }

        let models_folder = aeternum_config_dir_path.join("models");

        if !models_folder.exists() {
            debug!("Creating models directory for aeternum...");
            if let Err(err) = fs::create_dir_all(&models_folder) {
                return Err(
                    format!("Unable to create models path: {}", err).into()
                );
            };

            debug!("Models directory created!");
        }

        let toml_config_path = aeternum_config_dir_path.join("config.toml");

        if toml_config_path.exists() {
            debug!("Reading and applying config file...");
            let value = fs::read_to_string(&toml_config_path)?;

            let config = toml::from_str::<Config>(&value)?;
            return Ok(config);
        }

        debug!(
            "Reading template config and creating config file at '{}'...", 
            &toml_config_path.to_string_lossy().to_string()
        );
        let result = fs::write(
            &toml_config_path, include_bytes!("../../assets/config.template.toml")
        );

        match result {
            Ok(_) => Ok(
                toml::from_str(include_str!("../../assets/config.template.toml"))
                    .expect("Failed to deserialize template toml file!")
            ),
            Err(error) => {
                Err(
                    format!(
                        "Unable to create toml config at '{}'! Defaulting to default config. Error: {}",
                        toml_config_path.to_string_lossy().to_string(), error
                    ).into()
                )
            }
        }
    }
}