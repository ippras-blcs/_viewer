use egui::{Response, RichText, Ui, Widget};
use egui_l20n::{ResponseExt as _, UiExt as _};
use egui_phosphor::regular::{CHART_LINE, TABLE};
use serde::{Deserialize, Serialize};

/// View
#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Serialize)]
pub(crate) enum View {
    Plot,
    #[default]
    Table,
}

impl View {
    pub(crate) fn text(&self) -> &'static str {
        match self {
            Self::Plot => "plot",
            Self::Table => "table",
        }
    }

    pub(crate) fn hover_text(&self) -> &'static str {
        match self {
            Self::Plot => "plot.hover",
            Self::Table => "table.hover",
        }
    }
}

impl View {
    pub(crate) const fn icon(&self) -> &str {
        match self {
            Self::Plot => CHART_LINE,
            Self::Table => TABLE,
        }
    }
}

/// View widget
#[derive(Debug)]
pub(crate) struct ViewWidget<'a> {
    view: &'a mut View,
}

impl<'a> ViewWidget<'a> {
    pub(crate) fn new(view: &'a mut View) -> Self {
        Self { view }
    }
}

impl Widget for ViewWidget<'_> {
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
                ui.close_menu();
            }
        })
        .response
    }
}
