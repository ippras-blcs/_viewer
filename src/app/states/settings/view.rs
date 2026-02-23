use egui_phosphor::regular::{CHART_LINE, TABLE};
use serde::{Deserialize, Serialize};

/// View
#[derive(Clone, Copy, Debug, Default, Deserialize, Hash, PartialEq, Serialize)]
pub enum View {
    Plot,
    #[default]
    Table,
}

impl View {
    pub const fn icon(&self) -> &str {
        match self {
            Self::Plot => CHART_LINE,
            Self::Table => TABLE,
        }
    }

    pub fn text(&self) -> &'static str {
        match self {
            Self::Plot => "Plot",
            Self::Table => "Table",
        }
    }

    pub fn hover_text(&self) -> &'static str {
        match self {
            Self::Plot => "Plot.hover",
            Self::Table => "Table.hover",
        }
    }
}
