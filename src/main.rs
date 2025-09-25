#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use std::{env, fs, path::PathBuf, time::Duration};

use app::Aeternum;
use cirrus_path::v1::{get_user_config_dir_path};
use image::Image;
use log::debug;
use eframe::egui::{self, Style};
use egui_notify::ToastLevel;
use cirrus_theming::v1::Theme;
use cirrus_egui::v1::{config_manager::ConfigManager, notifier::Notifier, styling::Styling};
use clap::{arg, command, Parser};
use error::Error;

use config::config::Config;
use upscale::Upscale;

mod error;
mod app;
mod image;
mod windows;
mod files;
mod upscale;
mod config;

static APP_NAME: &str = "aeternum";
static TEMPLATE_CONFIG_TOML_STRING: &str = include_str!("../assets/config.template.toml");

#[derive(Parser, Debug)]
#[clap(author = "Ananas")]
#[command(version, about, long_about = None)]
struct Args {
    /// Valid path to image.
    image: Option<String>,

    /// Valid themes at the moment: dark, light
    #[arg(short, long)]
    theme: Option<String>,
}

fn main() -> eframe::Result {
    if !env::var("RUST_LOG").is_ok() {
        env::set_var("RUST_LOG", "WARN");
    }

    env_logger::init();

    let notifier = Notifier::new();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
        .with_inner_size([800.0, 600.0])
        .with_drag_and_drop(true),
        ..Default::default()
    };

    let cli_args = Args::parse();

    let image_path = cli_args.image;
    let theme_string = cli_args.theme;

    if image_path.is_some() {
        debug!("Using image: '{}'", &image_path.as_ref().unwrap());
    }

    let image = match image_path {
        Some(path) => {
            let path = PathBuf::from(&path);

            if !path.exists() {
                let error = Error::FileNotFound(
                    None,
                    path.to_path_buf(),
                    "That file doesn't exist!".to_string()
                );

                notifier.toast(
                    Box::new(error),
                    ToastLevel::Error,
                    |toast| {
                        toast.duration(Some(Duration::from_secs(10)));
                    }
                );

                None
            } else {
                match Image::from_path(path) {
                    Ok(image) => Some(image),
                    Err(error) => {
                        notifier.toast(
                            Box::new(error),
                            ToastLevel::Error,
                            |_| {}
                        );

                        None
                    }
                }
            }
        },
        None => None
    };

    let is_dark = match theme_string {
        Some(string) => {
            if string == "light" {
                false
            } else if string == "dark" {
                true
            } else {
                log::warn!(
                    "'{}' is not a valid theme. Pass either 'dark' or 'light'.", string
                );

                true
            }
        },
        _ => true
    };

    let theme = Theme::new(
        is_dark,
        vec![],
        None
    );

    let config_manager: ConfigManager<Config> = match ConfigManager::new(APP_NAME, TEMPLATE_CONFIG_TOML_STRING) {
        Ok(config) => config,
        Err(error) => {
            notifier.toast(
                format!("Failed to initialize config! Error: {}", error.human_message()), 
                ToastLevel::Error,
                |toast| {
                    toast.duration(Some(Duration::from_secs(10)));
                }
            );

            ConfigManager::default()
        }
    };

    match get_user_config_dir_path(APP_NAME) {
        Ok(config_dir_path) => {
            let models_folder = config_dir_path.join("models");

            if !models_folder.exists() {
                debug!("Creating models directory for aeternum...");

                match fs::create_dir_all(&models_folder) {
                    Ok(_) => debug!("Models directory created!"),
                    Err(error) => {
                        notifier.toast(
                            format!(
                                "Failed to create models directory! Error: {}", error
                            ), 
                            ToastLevel::Error,
                            |_| {}
                        );
                    },
                }
            }
        },
        Err(error) => {
            notifier.toast(
                format!(
                    "Failed to create models directory because we \
                    failed to get the user's config path! Error: {}", error.human_message()
                ),
                ToastLevel::Error,
                |_| {}
            );
        },
    };

    let mut upscale = match Upscale::new() {
        Ok(upscale) => upscale,
        Err(error) => {
            notifier.toast(
                Box::new(error.clone()),
                ToastLevel::Error,
                |_| {}
            );

            panic!("{}", error.to_string());
        }
    };

    match upscale.init(config_manager.config.misc.enable_custom_folder) {
        Ok(_) => {},
        Err(error) => {
            notifier.toast(
                Box::new(error.clone()),
                ToastLevel::Error,
                |_| {}
            );

            panic!("{}", error.to_string());
        }
    }

    eframe::run_native(
        "Aeternum",
        options,
        Box::new(|cc| {
            egui_extras::install_image_loaders(&cc.egui_ctx);

            let mut custom_style = Style {..Default::default()};

            custom_style.spacing.slider_width = 180.0;

            Styling::new(&theme, Some(custom_style))
                .set_all()
                .apply(&cc.egui_ctx);

            Ok(
                Box::new(
                    Aeternum::new(image, theme, notifier, upscale, config_manager)
                )
            )
        }),
    )
}