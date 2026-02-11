use egui::{Response, RichText, Ui, Widget};
use egui_l20n::prelude::*;
use egui_phosphor::regular::SLIDERS_HORIZONTAL;

/// Settings button widget
#[derive(Debug)]
pub struct SettingsButton<'a> {
    selected: &'a mut bool,
    size: Option<f32>,
}

impl<'a> SettingsButton<'a> {
    pub fn new(selected: &'a mut bool) -> Self {
        Self {
            selected,
            size: None,
        }
    }

    pub fn size(self, size: f32) -> Self {
        Self {
            size: Some(size),
            ..self
        }
    }
}

impl Widget for SettingsButton<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        let mut atoms = RichText::new(SLIDERS_HORIZONTAL);
        atoms = if let Some(size) = self.size {
            atoms.size(size)
        } else {
            atoms.heading()
        };
        ui.toggle_value(self.selected, atoms)
            .on_hover_localized("Settings")
    }
}
