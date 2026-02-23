use crate::app::states::settings::View;
use egui::{Response, RichText, Ui, Widget};
use egui_l20n::prelude::*;

/// View button
#[derive(Debug)]
pub(crate) struct ViewButton<'a> {
    view: &'a mut View,
}

impl<'a> ViewButton<'a> {
    pub(crate) fn new(view: &'a mut View) -> Self {
        Self { view }
    }
}

impl Widget for ViewButton<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        ui.menu_button(RichText::new(self.view.icon()).heading(), |ui| {
            let mut response = ui
                .selectable_value(
                    self.view,
                    View::Table,
                    format!("{} {}", View::Table.icon(), ui.localize(View::Table.text())),
                )
                .on_hover_localized(View::Table.hover_text());
            response |= ui
                .selectable_value(
                    self.view,
                    View::Plot,
                    format!("{} {}", View::Plot.icon(), ui.localize(View::Plot.text())),
                )
                .on_hover_localized(View::Plot.hover_text());
            if response.changed() {
                ui.close();
            }
        })
        .response
    }
}
