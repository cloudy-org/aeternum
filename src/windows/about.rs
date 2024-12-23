use cirrus_egui::v1::widgets::about::{authors_toml_to_about_authors, About, AboutApplicationInfo};
use eframe::egui::{self, Key, Response, Vec2};

use crate::files;

const VERSION: &str = env!("CARGO_PKG_VERSION");
const AUTHORS: &str = include_str!("../../authors.toml");

pub struct AboutWindow<'a> {
    pub show: bool,
    about_widget: About<'a>,
    pub response: Option<Response>,
}

impl<'a> AboutWindow<'a> {
    pub fn new() -> Self {
        let about_app_info = AboutApplicationInfo {
            name: "Aeternum".to_string(),
            description: "A simple and minimal upscaler built in rust".to_string(),
            license: include_str!("../../LICENSE").to_string(),
            version: VERSION.to_string(),
            authors: authors_toml_to_about_authors(&AUTHORS.to_string()),
            webpage: "https://github.com/cloudy-org/aeternum".to_string(),
            git_repo: "https://github.com/cloudy-org/aeternum".to_string(),
            copyright: "Copyright (C) 2024 Goldy".to_string()
        };

        let about_widget = About::new(
            files::get_aeternum_image(), about_app_info
        );

        Self {
            show: false,
            about_widget,
            response: None
        }
    }

    pub fn handle_input(&mut self, ctx: &egui::Context) {
        if ctx.input(|i| i.key_pressed(Key::A)) {
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