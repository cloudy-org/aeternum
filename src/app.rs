use cirrus_config::{config_key_path};
use cirrus_egui::v1::{config_manager::ConfigManager, notifier::Notifier, ui_utils::combo_box::{self}, widgets::settings::{section::{Section, SectionDisplayInfo, SectionOverrides}, Settings}};
use cirrus_theming::v1::Theme;
use eframe::egui::{self, Align, Color32, Context, CursorIcon, Frame, Layout, Margin, RichText, Slider, Vec2};
use egui::{include_image, Button, Key, OpenUrl, Sense, Stroke, UiBuilder};
use egui_notify::ToastLevel;
use strum::IntoEnumIterator;
use std::{time::Duration};

use crate::{config::config::Config, files, upscale::{OutputExt, Upscale}, windows::about::AboutWindow, Image, TEMPLATE_CONFIG_TOML_STRING};

pub struct Aeternum<'a> {
    theme: Theme,
    image: Option<Image>,
    about_box: AboutWindow<'a>,
    notifier: Notifier,
    upscale: Upscale,
    config_manager: ConfigManager<Config>,

    show_settings: bool,
}

impl<'a> Aeternum<'a> {
    pub fn new(image: Option<Image>, theme: Theme, notifier: Notifier, upscale: Upscale, config_manager: ConfigManager<Config>) -> Self {
        let about_box = AboutWindow::new(&config_manager.config, &notifier);

        Self {
            image,
            theme,
            notifier,
            about_box,
            upscale,
            config_manager,

            show_settings: false
        }
    }
}

impl<'a> eframe::App for Aeternum<'a> {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        self.about_box.handle_input(ctx);

        if ctx.input(|input| input.key_pressed(Key::Escape)) {
            self.show_settings = false;
        }

        // TODO: make this keybind customizable via the 
        // config in the future when we have good keybinds parsing.
        if ctx.input(|i| i.modifiers.ctrl && i.key_pressed(Key::Comma)) {
            self.show_settings = !self.show_settings;
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            self.upscale.update();
            self.notifier.update(ctx);
            self.about_box.update(ctx);

            if self.show_settings {
                // we only want to run the config manager's 
                // update loop when were are in the settings menu
                self.config_manager.update(ctx, &mut self.notifier);

                let config = &mut self.config_manager.config;

                Settings::new(TEMPLATE_CONFIG_TOML_STRING, &ui)
                    .add_section(
                        Section::new(
                            config_key_path!(config.misc.enable_custom_folder),
                            &mut config.misc.enable_custom_folder,
                            SectionOverrides::default(),
                            SectionDisplayInfo::default()
                        )
                    ).show_ui(ui, &self.theme);

                return;
            }

            let frame_margin = Margin {
                left: 15,
                right: 5,
                top: 14,
                bottom: 0, // let's leave the bottom with no padding until we add something there.
            };
            let side_panel_size: f32 = 240.0;

            egui::SidePanel::left("options_panel")
                .show_separator_line(true)
                .frame(Frame::default().outer_margin(frame_margin))
                .exact_width(side_panel_size + frame_margin.right as f32 + 1.0)
                .resizable(false)
                .show(ctx, |ui| {
                    ui.vertical_centered(|ui| {
                        let (_, rect) = ui.allocate_space(Vec2::new(ui.available_size().x, 35.0));
                        let mut child_ui = ui.new_child(UiBuilder::new().max_rect(rect).layout(Layout::default()));

                        let header_response = child_ui.horizontal_centered(|ui| {
                            let image = egui::Image::new(
                                include_image!("../assets/crystal_80x80.png")
                            ).fit_to_exact_size([35.0, 35.0].into());

                            ui.add(image);

                            ui.add_space(5.0);

                            ui.add(
                                egui::Label::new(
                                    RichText::new("Aeternum").size(25.0).strong()
                                ).selectable(false)
                            );
                        }).response;

                        if header_response.on_hover_cursor(CursorIcon::PointingHand).interact(Sense::click()).clicked() {
                            ui.ctx().open_url(
                                OpenUrl::new_tab("https://github.com/cloudy-org/aeternum")
                            );
                        }
                    });

                    ui.add_space(20.0);

                    // TODO: find a way to fix scrollbar slightly covering 
                    // elements if even possible, I've already spent hours fixing this :( 
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        ui.add_enabled_ui(!self.upscale.upscaling && !self.image.is_none(), |ui| {
                            egui::Grid::new("options_grid")
                                .spacing([0.0, 25.0])
                                //.max_col_width(side_panel_size)
                                .show(ui, |ui| {
                                    ui.vertical_centered_justified(|ui| {
                                        ui.label(RichText::new("Model").size(20.0).strong());
                                        ui.label(RichText::new("Select an upscaling model.").size(10.0));

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
                                                        combo_box::ui_strong_selectable_value(
                                                            ui,
                                                            &mut self.upscale.options.model,
                                                            Some(model.clone()),
                                                            &model.name
                                                        );
                                                    }
                                                });
                                        });
                                    });
                                    ui.end_row();

                                    ui.vertical_centered_justified(|ui| {
                                        let detailed_hint = "The higher you go the better the image quality is, UNTIL it isn't and \
                                        the model begins generating unwanted artifacts and distortion. Also the higher you go the bigger \
                                        your image size get's. Experiment with this scale on all images.";

                                        ui.label(RichText::new("Scale").size(20.0).strong());
                                        ui.label(RichText::new("The image resolution to upscale to.").size(10.0));
                                        ui.add(
                                            Slider::new(&mut self.upscale.options.scale, 1..=16)
                                        ).on_hover_text(detailed_hint).on_disabled_hover_text(detailed_hint);

                                        let scale = self.upscale.options.scale;

                                        let image_size = match &self.image {
                                            Some(image) => (image.image_size.width as i32, image.image_size.height as i32),
                                            None => (0, 0),
                                        };

                                        ui.label(
                                            format!("({}x{})", image_size.0 * scale as i32, image_size.1 * scale as i32)
                                        );
                                    });
                                    ui.end_row();

                                    ui.vertical_centered_justified(|ui| {
                                        ui.label(RichText::new("Compression").size(20.0).strong());
                                        ui.add(
                                            Slider::new(&mut self.upscale.options.compression, 0..=100)
                                        );
                                    });
                                    ui.end_row();

                                    ui.vertical_centered_justified(|ui| {
                                        ui.label(RichText::new("Image Format").size(20.0).strong());
                                        ui.label(RichText::new("Save image as...").size(10.0));

                                        let selected_ext = &self.upscale.options.output_ext.to_string();

                                        egui::ComboBox::from_id_salt("select_model")
                                            .selected_text(selected_ext)
                                            .width(230.0)
                                            .show_ui(ui, |ui| {
                                                for extension in OutputExt::iter() {
                                                    combo_box::ui_strong_selectable_value(
                                                        ui,
                                                        &mut self.upscale.options.output_ext,
                                                        extension.clone(),
                                                        extension.to_string()
                                                    );
                                                }
                                            });
                                    });
                                    ui.end_row();

                                    ui.vertical_centered_justified(|ui| {
                                        ui.label(RichText::new("Output Folder").size(20.0).strong());
                                        ui.label(RichText::new("Folder to drop your upscaled image.").size(10.0));

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
                                                    self.notifier.toast(
                                                        Box::new(error),
                                                        ToastLevel::Error,
                                                        |toast| {
                                                            toast.duration(Some(Duration::from_secs(5)));
                                                        }
                                                    );
                                                }
                                            }
                                        }
                                    });
                                    ui.end_row();

                                    ui.vertical_centered_justified(|ui| {
                                        let upscale_button = egui::Button::new(
                                            RichText::new("Upscale")
                                                .size(25.0)
                                            ).min_size([50.0, 60.0].into());
    
                                        let upscale_button_response = ui.add_enabled(
                                            self.upscale.options.model.is_some(), upscale_button
                                        ).on_disabled_hover_text("No model selected.")
                                        .on_hover_cursor(CursorIcon::PointingHand);

                                        if upscale_button_response.clicked() {
                                            self.upscale.upscale(self.image.clone().unwrap(), &self.notifier);
                                        }
                                    });
                                    ui.end_row();
                                });
                        });
                    });
                });

            let menu_bar_response = egui::TopBottomPanel::top("menu_bar")
                .show_separator_line(false)
                .frame(
                    Frame::new()
                        .inner_margin(Margin {right: 10, top: 10, ..1.into()})
                )
                .show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            if self.image.is_some() {
                                let button = egui::Button::new(
                                RichText::new("New Image").size(14.0)
                                ).min_size(Vec2::new(90.0, 25.0));
    
                                let response = ui.add(button)
                                    .on_hover_cursor(CursorIcon::PointingHand);
    
                                if response.clicked() {
                                    // self.upscale.reset_options();
                                    self.image = None;
                                }
                            }
                        });
                    });
                }).response;

            egui::CentralPanel::default().show(ctx, |ui| {
                match self.image.as_ref() {
                    Some(image) => {
                        let image_path = format!("file://{}", image.path.to_string_lossy());

                        ui.centered_and_justified(|ui| {
                            ui.add(
                                egui::Image::from_uri(image_path)
                                    .corner_radius(8.0)
                                    .shrink_to_fit()
                                    .max_size(
                                        [image.image_size.width as f32, image.image_size.height as f32].into()
                                    )
                            )
                        });
                    },
                    None => {
                        // Collect dropped files. UNTESTED!
                        ctx.input(|i| {
                            let dropped_files = &i.raw.dropped_files;

                            if !dropped_files.is_empty() {
                                let path = dropped_files.first()
                                    .unwrap()
                                    .path
                                    .as_ref()
                                    .unwrap(); // Umm I wonder why "PathBuf" is optional (Optional<T>) here.

                                match Image::from_path(path.clone()) {
                                    Ok(image) => self.image = Some(image),
                                    Err(error) => {
                                        self.notifier.toast(
                                            Box::new(error),
                                            ToastLevel::Error,
                                            |_| {}
                                        );
                                        return;
                                    }
                                };
                            }
                        });

                        ui.centered_and_justified(|ui| {
                            const SIZE_OF_VERTICAL_CENTRED: f32 = 251.0; // WARNING: changing anything under "ui.vertical_centered"
                            // will alter this size value so make sure you update it.

                            let file_is_hovering = !ctx.input(|i| i.raw.hovered_files.is_empty());

                            ui.add_space(((ui.available_height() + menu_bar_response.rect.height()) / 2.0) - SIZE_OF_VERTICAL_CENTRED / 1.7);

                            let vertical_centred_response = ui.vertical_centered(|ui| {
                                let image = egui::Image::new(
                                    include_image!("../assets/crystal_150x150.gif")
                                ).max_width(150.0);

                                ui.add(image);
                                ui.add_space(15.0);

                                let button = Button::new(
                                    RichText::new("Open Image")
                                        .size(25.0)
                                )
                                .min_size(Vec2::new(190.0, 60.0))
                                .corner_radius(20.0);

                                let button_response = ui.add(button);
                                button_response.clone().on_hover_cursor(CursorIcon::PointingHand);
                                //button_rect = button_response.rect;
                                ui.add_space(8.0);

                                let hint_message = match file_is_hovering {
                                    true => "Drop image to upscale it...",
                                    false => "Pick an image to upscale...",
                                };

                                ui.label(
                                    RichText::new(hint_message)
                                    .size(10.0)
                                );

                                if button_response.clicked() {
                                    let image_result = files::select_image();

                                    match image_result {
                                        Ok(image) => {
                                            self.image = Some(image);
                                            // I was able to get the memory of Aeternum to 
                                            // 500 MB by just loading a different image after another.
                                            // 
                                            // Using "ctx.forget_all_images()" should be okay for now as this 
                                            // also clears the "sparkles" gif that takes a very big amount of 
                                            // memory that I also want cleared from memory as we won't be 
                                            // seeing it again when we load an image.
                                            ctx.forget_all_images();
                                        },
                                        Err(error) => {
                                            self.notifier.toast(
                                                Box::new(error),
                                                ToastLevel::Error,
                                                |toast| {
                                                    toast.duration(Some(Duration::from_secs(5)));
                                                }
                                            );
                                        },
                                    }
                                }
                            }).response;

                            if file_is_hovering {
                                let mut rect = vertical_centred_response.rect.clone();
                                rect.set_width(190.0);

                                rect = rect.expand2(Vec2::new(150.0, 100.0));
                                rect.set_center(vertical_centred_response.rect.center());

                                let painter = ui.painter();

                                // Draw dotted lines to indicate file being dropped.
                                for index in 0..4 {
                                    let pos = match index {
                                        0 => &[rect.left_top(), rect.right_top()],
                                        1 => &[rect.right_top(), rect.right_bottom()],
                                        2 => &[rect.right_bottom(), rect.left_bottom()],
                                        3 => &[rect.left_bottom(), rect.left_top()],
                                        _ => unreachable!()
                                    };

                                    painter.add(
                                        egui::Shape::dashed_line(
                                            pos,
                                            Stroke {
                                                width: 2.0,
                                                color: Color32::from_hex(
                                                    &self.theme.accent_colour.hex_code
                                                ).unwrap()
                                            },
                                            11.0,
                                            10.0
                                        )
                                    );
                                }
                            }

                            assert_eq!(
                                vertical_centred_response.rect.height(),
                                SIZE_OF_VERTICAL_CENTRED,
                                "Some programmer did an idiot move \
                                (size of 'ui.vertical_centered' needs to be set again)..."
                            );
                        });
                    },
                }
            });
        });

        egui::TopBottomPanel::bottom("status_bar")
            .show_separator_line(false)
            .frame(
                Frame::NONE
                    .outer_margin(Margin {right: 12, bottom: 8, ..Default::default()})
            ).show(ctx, |ui| {
                if let Some(loading) = &self.notifier.loading {
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
            });
    }
}
