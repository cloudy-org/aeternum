use cirrus_egui::v1::widgets::about::{authors_toml_to_about_authors, About, AboutApplicationInfo};
use eframe::egui::{self, Key, Response, Vec2};
use egui::include_image;
use egui_notify::ToastLevel;

use crate::{config::config::Config, notifier::NotifierAPI};

const VERSION: &str = env!("CARGO_PKG_VERSION");
const AUTHORS: &str = include_str!("../../authors.toml");

pub struct AboutWindow<'a> {
    pub show: bool,
    about_widget: About<'a>,
    toggle_key: Key,
    pub response: Option<Response>,
}

impl<'a> AboutWindow<'a> {
    pub fn new(config: &Config, notifier: &mut NotifierAPI) -> Self {        
        let config_key = match Key::from_name(&config.keybinds.about_box) {
            Some(key) => key,
            None => {
                notifier.toasts.lock().unwrap().toast_and_log(
                    "The key bind set for 'about_box' is invalid! Defaulting to `A`.".into(), 
                    ToastLevel::Error
                );

                Key::A
            },
        };

        let about_app_info = AboutApplicationInfo {
            name: "Aeternum".to_string(),
            description: "A simple and minimal upscaler built in rust".to_string(),
            license: include_str!("../../LICENSE").to_string(),
            version: VERSION.to_string(),
            authors: authors_toml_to_about_authors(&AUTHORS.to_string()),
            webpage: "https://github.com/cloudy-org/aeternum".to_string(),
            git_repo: "https://github.com/cloudy-org/aeternum".to_string(),
            copyright: "Copyright (C) 2024 Ananas".to_string()
        };

        let about_widget = About::new(
            include_image!("../../assets/sparkles.png"), about_app_info
        );

        Self {
            show: false,
            about_widget,
            toggle_key: config_key,
            response: None
        }
    }

    pub fn handle_input(&mut self, ctx: &egui::Context) {
        if ctx.input(|i| i.key_pressed(self.toggle_key)) {
            if self.show == true {
                self.show = false;
            } else {
                self.show = true;
            }
        }
    }

    pub fn update(&mut self, ctx: &egui::Context) {
        if self.show {
            let default_window_size = Vec2::new(340.0, 350.0);

            let response = egui::Window::new(
                egui::WidgetText::RichText(
                    egui::RichText::new("ℹ About").size(15.0)
                )
            )
                .default_size(default_window_size)
                .min_width(270.0)
                .default_pos(ctx.screen_rect().center() - default_window_size / 2.0)
                .show(ctx, |ui| {
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        self.about_widget.show(ctx, ui);
                    });
                });

            self.response = Some(response.unwrap().response);
        }

        self.about_widget.update(ctx);
    }
}