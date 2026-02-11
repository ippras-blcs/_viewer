use egui::{Response, RichText, Ui, Widget};
use egui_l20n::prelude::*;
use egui_phosphor::regular::SIDEBAR_SIMPLE;

/// Left panel button widget
#[derive(Debug)]
pub struct LeftPanelButton<'a> {
    selected: &'a mut bool,
    size: Option<f32>,
}

impl<'a> LeftPanelButton<'a> {
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

impl Widget for LeftPanelButton<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        let mut atoms = RichText::new(SIDEBAR_SIMPLE);
        atoms = if let Some(size) = self.size {
            atoms.size(size)
        } else {
            atoms.heading()
        };
        ui.toggle_value(self.selected, atoms)
            .on_hover_localized("LeftPanel")
    }
}
