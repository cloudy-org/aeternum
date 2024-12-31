use cirrus_theming::v1::{Colour, Theme};
use eframe::egui::{self, Align, Color32, Context, CursorIcon, Frame, Id, Layout, Margin, Rect, RichText, Shadow, Slider, Stroke, Style, TextStyle, Vec2};
use egui_notify::ToastLevel;
use std::time::Duration;

use crate::{config::config::Config, files, notifier::NotifierAPI, upscale::Upscale, windows::about::AboutWindow, Image};

pub struct Aeternum<'a> {
    theme: Theme,
    image: Option<Image>,
    about_box: AboutWindow<'a>,
    notifier: NotifierAPI,
    upscale: Upscale
}

impl<'a> Aeternum<'a> {
    pub fn new(image: Option<Image>, theme: Theme, mut notifier: NotifierAPI, upscale: Upscale, config: Config) -> Self {
        let about_box = AboutWindow::new(&config, &mut notifier);

        Self {
            image,
            theme,
            notifier,
            about_box,
            upscale
        }
    }

    fn set_app_style(&self, ctx: &Context) {
        let mut custom_style = Style {
            override_text_style: Some(TextStyle::Monospace),
            ..Default::default()
        };

        custom_style.visuals.panel_fill = Color32::from_hex(
            &self.theme.primary_colour.hex_code
        ).unwrap();

        // Window styling.
        custom_style.visuals.window_highlight_topmost = false;

        custom_style.visuals.window_fill = Color32::from_hex(
            &self.theme.secondary_colour.hex_code
        ).unwrap();
        custom_style.visuals.window_stroke = Stroke::new(
            1.0,
            Color32::from_hex(&self.theme.third_colour.hex_code).unwrap()
        );
        custom_style.visuals.window_shadow = Shadow::NONE;

        custom_style.visuals.widgets.inactive.bg_fill =
            Color32::from_hex(
                &self.theme.primary_colour.hex_code
            ).unwrap();

        // Text styling.
        custom_style.visuals.override_text_color = Some(
            Color32::from_hex(
                &self.theme.text_colour.hex_code
            ).unwrap()
        );

        custom_style.visuals.slider_trailing_fill = true;

        custom_style.spacing.slider_width = 180.0;

        ctx.set_style(custom_style);
    }

    fn draw_dotted_line(&self, ui: &egui::Painter, pos: &[egui::Pos2]) {
        ui.add(
            egui::Shape::dashed_line(
                pos,
                Stroke {
                    width: 2.0,
                    color: Color32::from_hex(
                        &self.theme.accent_colour.as_ref()
                            .unwrap_or(&Colour {hex_code: "e05f78".into()}).hex_code
                    ).unwrap()
                },
                10.0,
                10.0
            )
        );
    }
}

impl eframe::App for Aeternum<'_> {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        self.set_app_style(ctx);

        self.about_box.handle_input(ctx);
        self.upscale.update();

        egui::CentralPanel::default().show(ctx, |ui| {
            let window_rect = ctx.input(|i: &egui::InputState| i.screen_rect());

            self.notifier.update(ctx);
            self.about_box.update(ctx);

            if self.image.is_none() {
                // Collect dropped files.
                ctx.input(|i| {
                    let dropped_files = &i.raw.dropped_files;

                    if !dropped_files.is_empty() {
                        let path = dropped_files.first().unwrap()
                            .path
                            .as_ref()
                            .unwrap();

                        match Image::from_path(path.clone()) {
                            Ok(image) => self.image = Some(image),
                            Err(error) => {
                                self.notifier.toasts.lock().unwrap().toast_and_log(
                                    error.into(), ToastLevel::Error
                                );
                                return;
                            }
                        };
                    }
                });

                ui.centered_and_justified(|ui| {
                    let image_width: f32 = 145.0;
                    let file_is_hovering = !ctx.input(|i| i.raw.hovered_files.is_empty());

                    let mut aeter_rect = Rect::NOTHING;

                    egui::Frame::default()
                        .outer_margin(
                            Margin::symmetric(
                                (window_rect.width() / 2.0) - image_width / 2.0,
                                (window_rect.height() / 2.0) - image_width / 2.0
                            )
                        )
                        .show(ui, |ui| {
                            let aeter_response = ui.add(
                                egui::Image::new(files::get_aeternum_image())
                                    .max_width(image_width)
                                    .sense(egui::Sense::click())
                            );

                            aeter_rect = aeter_response.rect;

                            if file_is_hovering {
                                ui.label("You're about to drop a file.");
                            }

                            aeter_response.clone().on_hover_cursor(CursorIcon::PointingHand);

                            if aeter_response.clicked() {
                                let image_result = files::select_image();

                                match image_result {
                                    Ok(image) => self.image = Some(image),
                                    Err(error) => {
                                        self.notifier.toasts.lock().unwrap()
                                            .toast_and_log(error.into(), ToastLevel::Error)
                                            .duration(Some(Duration::from_secs(5)));
                                    },
                                }
                            }
                        }
                    );

                    if file_is_hovering {
                        let rect = aeter_rect.expand2(
                            Vec2::new(150.0, 100.0)
                        );
                        let painter = ui.painter();

                        let top_right = rect.right_top();
                        let top_left = rect.left_top();
                        let bottom_right = rect.right_bottom();
                        let bottom_left = rect.left_bottom();

                        self.draw_dotted_line(painter, &[top_left, top_right]);
                        self.draw_dotted_line(painter, &[top_right, bottom_right]);
                        self.draw_dotted_line(painter, &[bottom_right, bottom_left]);
                        self.draw_dotted_line(painter, &[bottom_left, top_left]);
                    }
                });

                return;
            }

            let image = self.image.as_ref().unwrap();
            let side_panel_size = 240.0;

            egui::SidePanel::left("options_panel")
                .show_separator_line(false)
                .exact_width(side_panel_size)
                .resizable(false)
                .show(ctx, |ui| {
                        egui::Grid::new("options_grid")
                            .spacing([20.0, 45.0])
                            .show(ui, |ui| {
                            ui.vertical_centered_justified(|ui| {
                                ui.label("Model");

                                let selected = match &self.upscale.options.model {
                                    Some(model) => model.name.clone(),
                                    None => "Select a Model".to_string(),
                                };

                                ui.vertical_centered(|ui| {
                                    egui::ComboBox::from_id_salt("select_model")
                                        .selected_text(selected)
                                        .width(230.0)
                                        .show_ui(ui, |ui| {
                                            for model in self.upscale.models.iter() {
                                                ui.selectable_value(
                                                    &mut self.upscale.options.model,
                                                    Some(model.clone()),
                                                    model.name.to_string()
                                                );
                                            }
                                        });
                                });
                            });
                            ui.end_row();

                            ui.vertical_centered_justified(|ui| {
                                ui.label("Scale");
                                ui.add(
                                    Slider::new(&mut self.upscale.options.scale, 1..=16)
                                );

                                let scale = self.upscale.options.scale;
                                let width = image.image_size.width as i32;
                                let height = image.image_size.height as i32;

                                ui.label(format!("({}x{})", width * scale, height * scale));
                            });
                            ui.end_row();

                            ui.vertical_centered_justified(|ui| {
                                ui.label("Compression");
                                ui.add(
                                    Slider::new(&mut self.upscale.options.compression, 0..=100)
                                );
                            });
                            ui.end_row();

                            ui.vertical_centered_justified(|ui| {
                                ui.label("Output file");

                                let output_button = match &self.upscale.options.output {
                                    Some(path) => ui.button(path.to_str().unwrap()),
                                    None => {
                                        let model = self.upscale.options.model.is_some();

                                        ui.add_enabled(
                                            model,
                                            egui::Button::new("Select output")
                                        ).on_disabled_hover_text("Select a model before setting the output file.")
                                    }
                                };

                                if output_button.clicked() {
                                    match files::save_image(&image, &self.upscale.options) {
                                        Ok(output) => self.upscale.options.output = Some(output),
                                        Err(error) => {
                                            self.notifier.toasts.lock().unwrap()
                                                .toast_and_log(error.into(), ToastLevel::Error)
                                                .duration(Some(Duration::from_secs(5)));
                                        }
                                    }
                                }
                            });
                            ui.end_row();

                            let (button_enabled, disabled_text) = match (self.upscale.upscaling, self.upscale.options.model.is_some()) {
                                (_, false) => (false, "No model selected."),
                                (true, _) => (false, "Currently upscaling."),
                                _ => (true, "")
                            };

                            ui.vertical_centered_justified(|ui| {
                                let upscale_button = ui.add_enabled(
                                    button_enabled,
                                    egui::Button::new(RichText::new("Upscale").size(20.0))
                                        .min_size([50.0, 60.0].into())
                                ).on_disabled_hover_text(disabled_text);

                                if upscale_button.clicked() {
                                    self.upscale.upscale(image.clone(), &mut self.notifier);
                                }
                            });
                        });
                    });

            let area_width = (window_rect.width() / 2.0) - side_panel_size / 2.5;
            let area_height = (window_rect.height() / 2.0) - side_panel_size / 2.0;

            egui::Area::new(Id::new("image_area"))
                .fixed_pos([area_width, area_height])
                .show(ctx, |ui| {
                    let image_path = format!("file://{}", image.path.to_string_lossy());
                    ui.add(
                        egui::Image::from_uri(image_path)
                            .rounding(4.0)
                            .max_width(ui.available_width())
                    )
                });

            ctx.request_repaint_after_secs(1.0);
        });

        egui::TopBottomPanel::top("menu_bar")
            .show_separator_line(false)
            .frame(
                Frame::none()
                    .outer_margin(Margin {right: 10.0, top: 7.0, ..Default::default()})
            )
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if self.image.is_some() {
                            let exit_button =
                                ui.add(
                                    egui::Button::new("<-")
                                );

                            if exit_button.clicked() {
                                self.upscale.reset_options();
                                self.image = None;
                            }
                        }
                    });
                });
            });

        egui::TopBottomPanel::bottom("status_bar")
            .show_separator_line(false)
            .frame(
                Frame::none()
                    .outer_margin(Margin {right: 12.0, bottom: 8.0, ..Default::default()})
            ).show(ctx, |ui| {
                if let Ok(loading_status) = self.notifier.loading_status.try_read() {
                    if let Some(loading) = loading_status.as_ref() {
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            if let Some(message) = &loading.message {
                                ui.label(message);
                            }

                            ui.add(
                                egui::Spinner::new()
                                    .color(Color32::from_hex("#e05f78").unwrap()) // NOTE: This should be the default accent colour.
                                    .size(20.0)
                            );
                        });
                    }
                }
            });
    }
}
