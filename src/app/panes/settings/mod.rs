pub(crate) use self::table::{Order, Sort};

use self::{plot::Settings as PlotSettings, table::Settings as TableSettings};
use arrow::temporal_conversions::timestamp_ms_to_datetime;
use chrono::{DateTime, FixedOffset, Local, Offset as _, TimeZone as _, Utc};
use egui::{ComboBox, Grid, Ui};
use egui_l20n::{ResponseExt as _, UiExt as _};
use serde::{Deserialize, Serialize};

/// Settings
#[derive(Clone, Debug, Default, Deserialize, Hash, PartialEq, Serialize)]
pub(crate) struct Settings {
    pub(crate) time_zone: TimeZone,
    pub(crate) plot: PlotSettings,
    pub(crate) table: TableSettings,
}

impl Settings {
    pub(crate) const fn new() -> Self {
        Self {
            time_zone: TimeZone::Local,
            plot: PlotSettings::new(),
            table: TableSettings::new(),
        }
    }
}

// temperature: Temperature {
//     unit: TemperatureUnit::DegreeCelsius,
// },
// concentration: Concentration {
//     unit: ConcentrationUnit::MilligramPerCubicMeter,
// },
// temperature: Temperature {
//     unit: TemperatureUnit::DegreeCelsius,
// },
// concentration: Concentration {
//     unit: ConcentrationUnit::MilligramPerCubicMeter,
// },
//
// /// Concentration
// #[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
// pub(crate) struct Concentration {
//     pub(crate) unit: ConcentrationUnit,
// }
//
// /// Temperature
// #[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
// pub(crate) struct Temperature {
//     pub(crate) unit: TemperatureUnit,
// }

impl Settings {
    pub(crate) fn show(&mut self, ui: &mut Ui) {
        // ui.horizontal(|ui| {
        //     ui.label("Temperature unit:");
        //     ComboBox::from_id_source("temperature_unit")
        //         .selected_text(context.settings.temperature.unit.to_string())
        //         .show_ui(ui, |ui| {
        //             for unit in TEMPERATURE_UNITS {
        //                 ui.selectable_value(
        //                     &mut context.settings.temperature.unit,
        //                     unit,
        //                     unit.abbreviation(),
        //                 )
        //                 .on_hover_text(unit.singular());
        //             }
        //         })
        //         .response
        //         .on_hover_text(context.settings.temperature.unit.singular());
        // });
        // ui.horizontal(|ui| {
        //     ui.label("Concentration unit:");
        //     ComboBox::from_id_source("concentration_unit")
        //         .selected_text(context.settings.concentration.unit.abbreviation())
        //         .show_ui(ui, |ui| {
        //             for unit in CONCENTRATION_UNITS {
        //                 ui.selectable_value(
        //                     &mut context.settings.concentration.unit,
        //                     unit,
        //                     unit.abbreviation(),
        //                 )
        //                 .on_hover_text(unit.singular());
        //             }
        //         })
        //         .response
        //         .on_hover_text(context.settings.concentration.unit.singular());
        // });
        Grid::new(ui.next_auto_id()).show(ui, |ui| {
            // Time zone
            ui.label(ui.localize("time_zone"));
            ComboBox::from_id_salt("time_zone")
                .selected_text(ui.localize(self.time_zone.text()))
                .show_ui(ui, |ui| {
                    ui.selectable_value(
                        &mut self.time_zone,
                        TimeZone::Utc,
                        ui.localize(TimeZone::Utc.text()),
                    )
                    .on_hover_localized(TimeZone::Utc.hover_text());
                    ui.selectable_value(
                        &mut self.time_zone,
                        TimeZone::Local,
                        ui.localize(TimeZone::Local.text()),
                    )
                    .on_hover_localized(TimeZone::Local.hover_text());
                })
                .response
                .on_hover_localized(self.time_zone.hover_text());
            ui.end_row();
        });
    }
}

/// Offset
#[derive(Clone, Copy, Debug, Default, Deserialize, Hash, PartialEq, Serialize)]
pub enum TimeZone {
    Local,
    #[default]
    Utc,
}

impl TimeZone {
    fn text(&self) -> &'static str {
        match self {
            Self::Utc => "time_zone__utc",
            Self::Local => "time_zone__local",
        }
    }

    fn hover_text(&self) -> &'static str {
        match self {
            Self::Utc => "time_zone__utc.hover",
            Self::Local => "time_zone__local.hover",
        }
    }
}

impl TimeZone {
    pub(crate) fn format_time(&self, value: i64, format: &str) -> String {
        self.time(value).format(format).to_string()
    }

    pub(crate) fn offset(&self) -> FixedOffset {
        match self {
            TimeZone::Utc => Utc.fix(),
            TimeZone::Local => *Local::now().offset(),
        }
    }

    pub(crate) fn time(&self, value: i64) -> DateTime<FixedOffset> {
        let offset = self.offset();
        timestamp_ms_to_datetime(value)
            .map(|date_time| offset.from_utc_datetime(&date_time))
            .unwrap_or_else(|| Utc::now().with_timezone(&offset))
    }
}

mod plot;
mod table;
