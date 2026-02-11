use egui::{Response, RichText, Ui, Widget};
use egui_l20n::prelude::*;
use egui_phosphor::regular::ROCKET;

/// Reactive button widget
#[derive(Debug)]
pub struct ReactiveButton<'a> {
    selected: &'a mut bool,
    size: Option<f32>,
}

impl<'a> ReactiveButton<'a> {
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

impl Widget for ReactiveButton<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        let mut atoms = RichText::new(ROCKET);
        atoms = if let Some(size) = self.size {
            atoms.size(size)
        } else {
            atoms.heading()
        };
        ui.toggle_value(self.selected, atoms)
            .on_hover_localized("Reactive")
            .on_hover_localized("Reactive.hover?State=enabled")
            .on_disabled_hover_localized("Reactive.hover?State=disabled")
    }
}
