use cirrus_egui::v1::widgets::settings::Settings;
use cirrus_theming::v1::Theme;
use egui::{Key, Ui};

pub struct SettingsView {
    settings_widget: Settings,

    pub show: bool,
}

impl SettingsView {
    pub fn new() -> Self {
        Self {
            settings_widget: Settings::new(),
            show: false
        }
    }

    pub fn handle_input(&mut self, ctx: &egui::Context) {
        if ctx.input(|input| input.key_pressed(Key::Escape)) {
            self.show = false;
        }

        // TODO: make this keybind customizable via the 
        // config in the future when we have good keybinds parsing.
        if ctx.input(|i| i.modifiers.ctrl && i.key_pressed(Key::Comma)) {
            self.show = !self.show;
        }
    }

    pub fn show(&mut self, ctx: &egui::Context, ui: &mut Ui, theme: &Theme) {
        self.settings_widget.show(ctx, ui, theme);
    }
}