use crate::app::MAX_PRECISION;
use egui::{Slider, Ui, Widget as _};
use egui_ext::LabeledSeparator;
use egui_l20n::prelude::*;
use egui_phosphor::regular::BOOKMARK;
use serde::{Deserialize, Serialize};

/// Configuration settings
#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) struct Settings {
    pub(crate) index: usize,

    pub(crate) edit: bool,
    pub(crate) precision: usize,
    pub(crate) resizable: bool,
    pub(crate) sticky_columns: usize,
    pub(crate) truncate: bool,

    // Hover
    pub(crate) hover_names: bool,
    pub(crate) hover_properties: bool,
}

impl Settings {
    pub(crate) fn new() -> Self {
        Self {
            index: 0,

            edit: false,
            resizable: false,

            precision: 0,
            sticky_columns: 0,
            truncate: false,

            // Hover
            hover_names: true,
            hover_properties: true,
        }
    }
}

impl Settings {
    pub(crate) fn show(&mut self, ui: &mut Ui) {
        // ui.visuals_mut().button_frame = true;
        self.precision(ui);
        self.sticky_columns(ui);
        self.truncate(ui);

        // Hover
        ui.labeled_separator("Hover");

        self.hover_names(ui);
        self.hover_properties(ui);
    }

    // Precision
    fn precision(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label(ui.localize("Precision"))
                .on_hover_localized("Precision.hover");
            ui.horizontal(|ui| {
                Slider::new(&mut self.precision, 0..=MAX_PRECISION).ui(ui);
                if ui.button((BOOKMARK, "3")).clicked() {
                    self.precision = 3;
                }
            });
        });
    }

    /// Sticky columns
    fn sticky_columns(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label(ui.localize("StickyColumns"))
                .on_hover_localized("StickyColumns.hover");
            Slider::new(&mut self.sticky_columns, 0..=5).ui(ui);
        });
    }

    /// Truncate headers
    fn truncate(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label(ui.localize("TruncateHeaders"))
                .on_hover_localized("TruncateHeaders.hover");
            ui.checkbox(&mut self.truncate, ());
        });
    }

    /// Names
    fn hover_names(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label(ui.localize("Names"))
                .on_hover_localized("Names.hover");
            ui.checkbox(&mut self.hover_names, "");
        });
    }

    /// Properties
    fn hover_properties(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label(ui.localize("Properties"))
                .on_hover_localized("Properties.hover");
            ui.checkbox(&mut self.hover_properties, "");
        });
    }
}

impl Default for Settings {
    fn default() -> Self {
        Self::new()
    }
}

// #[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Serialize)]
// pub(crate) enum Shema {
//     TotalLipids,
//     StereospecificNumbers1223,
//     StereospecificNumbers13,
//     #[default]
//     StereospecificNumbers2,
// }

// impl Shema {
//     pub(crate) fn text(&self) -> &'static str {
//         match self {
//             Self::TotalLipids => "TotalLipids",
//             Self::StereospecificNumbers1223 => "StereospecificNumber?number=123",
//             Self::StereospecificNumbers13 => "StereospecificNumber?number=13",
//             Self::StereospecificNumbers2 => "StereospecificNumber?number=2",
//         }
//     }

//     pub(crate) fn hover_text(&self) -> &'static str {
//         match self {
//             Self::TotalLipids => "TotalLipids",
//             Self::StereospecificNumbers1223 => "StereospecificNumber.hover?number=123",
//             Self::StereospecificNumbers13 => "StereospecificNumber.hover?number=13",
//             Self::StereospecificNumbers2 => "StereospecificNumber.hover?number=2",
//         }
//     }
// }
