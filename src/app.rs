use cirrus_theming::v1::Theme;
use eframe::egui::{self, Align, Color32, Context, CursorIcon, Frame, Layout, Margin, Rect, RichText, Slider, Stroke, Vec2};
use egui_notify::ToastLevel;
use strum::IntoEnumIterator;
use std::time::Duration;

use crate::{config::config::Config, files, notifier::NotifierAPI, upscale::{OutputExt, Upscale}, windows::about::AboutWindow, Image};

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

    fn draw_dotted_line(&self, ui: &egui::Painter, pos: &[egui::Pos2]) {
        ui.add(
            egui::Shape::dashed_line(
                pos,
                Stroke {
                    width: 2.0,
                    color: Color32::from_hex(
                        &self.theme.accent_colour.hex_code
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
                                ((window_rect.width() / 2.0) - image_width / 2.0) as i8,
                                ((window_rect.height() / 2.0) - image_width / 2.0) as i8
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
                .show_separator_line(true)
                .exact_width(side_panel_size)
                .resizable(false)
                .show(ctx, |ui| {
                    ui.add_enabled_ui(!self.upscale.upscaling, |ui| {
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
                                    ui.label("Save image as");

                                    let selected_ext = &self.upscale.options.output_ext.to_string();

                                    egui::ComboBox::from_id_salt("select_model")
                                        .selected_text(selected_ext)
                                        .width(230.0)
                                        .show_ui(ui, |ui| {
                                            for extension in OutputExt::iter() {
                                                ui.selectable_value(
                                                    &mut self.upscale.options.output_ext,
                                                    extension.clone(),
                                                    extension.to_string()
                                                );
                                            }
                                        });
                                });
                                ui.end_row();

                                ui.vertical_centered_justified(|ui| {
                                    ui.label("Output folder");

                                    let output_button = match &self.upscale.options.output {
                                        Some(path) => ui.button(path.to_str().unwrap()),
                                        None => {
                                            let model = self.upscale.options.model.is_some();

                                            ui.add_enabled(
                                                model,
                                                egui::Button::new("Select output")
                                            ).on_disabled_hover_text("Select a model before setting the output folder.")
                                        }
                                    };

                                    if output_button.clicked() {
                                        match files::save_folder() {
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

                                let (button_enabled, disabled_text) = match self.upscale.options.model.is_some() {
                                    false => (false, "No model selected."),
                                    true => (true, "")
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
                
                });

            egui::CentralPanel::default()
                .show(ctx, |ui| {
                    let image_path = format!("file://{}", image.path.to_string_lossy());

                    ui.centered_and_justified(|ui| {
                        ui.add(
                            egui::Image::from_uri(image_path)
                                .rounding(4.0)
                                .shrink_to_fit()
                                .max_size(
                                    [image.image_size.width as f32, image.image_size.height as f32].into()
                                )
                        )
                    });
                });

            ctx.request_repaint_after_secs(1.0);
        });

        egui::TopBottomPanel::top("menu_bar")
            .show_separator_line(false)
            .frame(
                Frame::NONE
                    .outer_margin(Margin {right: 10, top: 7, ..Default::default()})
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
                Frame::NONE
                    .outer_margin(Margin {right: 12, bottom: 8, ..Default::default()})
            ).show(ctx, |ui| {
                if let Ok(loading_status) = self.notifier.loading_status.try_read() {
                    if let Some(loading) = loading_status.as_ref() {
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            if let Some(message) = &loading.message {
                                ui.label(message);
                            }

                            ui.add(
                                egui::Spinner::new()
                                    .color(
                                        Color32::from_hex(
                                            &self.theme.accent_colour.hex_code
                                        ).unwrap()
                                    )
                                    .size(20.0)
                            );
                        });
                    }
                }
            });
    }
}
