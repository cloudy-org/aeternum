#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use std::{env, path::PathBuf, time::Duration};

use app::Aeternum;
use image::Image;
use log::debug;
use eframe::egui::{self, Style};
use egui_notify::ToastLevel;
use cirrus_theming::v1::Theme;
use cirrus_egui::v1::styling::Styling;
use clap::{arg, command, Parser};
use error::Error;

use config::config::Config;
use notifier::NotifierAPI;
use upscale::Upscale;

mod error;
mod notifier;
mod app;
mod image;
mod windows;
mod files;
mod upscale;
mod config;

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

    let notifier = NotifierAPI::new();

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

                notifier.toasts.lock().unwrap().toast_and_log(
                    error.into(), ToastLevel::Error
                ).duration(Some(Duration::from_secs(10)));

                None
            } else {
                match Image::from_path(path) {
                    Ok(image) => Some(image),
                    Err(error) => {
                        notifier.toasts.lock().unwrap().toast_and_log(
                            error.into(), ToastLevel::Error
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

    let config = match Config::new() {
        Ok(config) => config,
        Err(error) => {

            notifier.toasts.lock().unwrap().toast_and_log(
                format!(
                    "Error occurred getting aeternum's config file! \
                    Defaulting to default config. Error: {}", error.to_string().as_str()
                ).into(), 
                ToastLevel::Error
            ).duration(Some(Duration::from_secs(10)));

            Config::default()
        }
    };

    let mut upscale = match Upscale::new() {
        Ok(upscale) => upscale,
        Err(error) => {
            notifier.toasts.lock().unwrap().toast_and_log(
                error.clone().into(), ToastLevel::Error
            );

            panic!("{}", error.clone().to_string());
        }
    };

    match upscale.init(config.misc.enable_custom_folder) {
        Ok(_) => {},
        Err(error) => {
            notifier.toasts.lock().unwrap().toast_and_log(
                error.clone().into(), ToastLevel::Error
            );

            panic!("{}", error.clone().to_string());
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
                    Aeternum::new(image, theme, notifier, upscale, config)
                )
            )
        }),
    )
}