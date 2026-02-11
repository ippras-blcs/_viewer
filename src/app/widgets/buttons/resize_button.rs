use egui::{Response, RichText, Ui, Widget};
use egui_l20n::prelude::*;
use egui_phosphor::regular::ARROWS_HORIZONTAL;

/// Resize button widget
#[derive(Debug)]
pub struct ResizeButton<'a> {
    selected: &'a mut bool,
    size: Option<f32>,
}

impl<'a> ResizeButton<'a> {
    pub fn new(selected: &'a mut bool) -> Self {
        Self {
            selected,
            size: None,
        }
    }

    #[allow(unused)]
    pub fn size(self, size: f32) -> Self {
        Self {
            size: Some(size),
            ..self
        }
    }
}

impl Widget for ResizeButton<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        let mut atoms = RichText::new(ARROWS_HORIZONTAL);
        atoms = if let Some(size) = self.size {
            atoms.size(size)
        } else {
            atoms.heading()
        };
        ui.toggle_value(self.selected, atoms)
            .on_hover_localized("ResizeTableColumns")
    }
}
