use chrono::{FixedOffset, Local, Offset as _, TimeZone, Utc};
use egui::{Color32, ComboBox, DragValue, Grid, RichText, Ui, Vec2b, emath::Float};
use egui_l20n::{ResponseExt, UiExt};
use serde::{Deserialize, Serialize};
use std::{
    borrow::Cow,
    hash::{Hash, Hasher},
};
use time::{UtcOffset, macros::offset};

/// Settings
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
pub(crate) struct Settings {
    pub(crate) drag: Vec2b,
    pub(crate) legend: bool,
    pub(crate) link: Vec2b,
    pub(crate) scroll: bool,
    pub(crate) zoom: Vec2b,
    // pub(crate) temperature: Temperature,
    // pub(crate) concentration: Concentration,
    pub(crate) time: Time,

    pub(crate) source: Source,
    pub(crate) resampling: Resampling,
    pub(crate) rolling: Rolling,
}

impl Settings {
    pub(crate) const fn new() -> Self {
        Self {
            drag: Vec2b { x: false, y: true },
            legend: true,
            link: Vec2b { x: false, y: false },
            scroll: false,
            zoom: Vec2b { x: false, y: true },
            time: Time {
                offset: UtcOffset::UTC,
                _offset: Offset::Local,
            },
            // temperature: Temperature {
            //     unit: TemperatureUnit::DegreeCelsius,
            // },
            // concentration: Concentration {
            //     unit: ConcentrationUnit::MilligramPerCubicMeter,
            // },
            source: Source::new(),
            resampling: Resampling::new(),
            rolling: Rolling::new(),
        }
    }
}

// temperature: Temperature {
//     unit: TemperatureUnit::DegreeCelsius,
// },
// concentration: Concentration {
//     unit: ConcentrationUnit::MilligramPerCubicMeter,
// },
impl Settings {
    pub(crate) fn show(&mut self, ui: &mut Ui) {
        ui.collapsing(RichText::new(ui.localize("plot")).heading(), |ui| {
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
                    .selected_text(ui.localize(self.time._offset.text()))
                    .show_ui(ui, |ui| {
                        ui.selectable_value(
                            &mut self.time._offset,
                            Offset::Utc,
                            ui.localize(Offset::Utc.text()),
                        )
                        .on_hover_text(ui.localize(Offset::Utc.hover_text()));
                        ui.selectable_value(
                            &mut self.time._offset,
                            Offset::Local,
                            ui.localize(Offset::Local.text()),
                        )
                        .on_hover_text(ui.localize(Offset::Local.hover_text()));
                    })
                    .response
                    .on_hover_localized(self.time._offset.hover_text());
                ui.end_row();
                // Legend
                ui.label(ui.localize("legend"));
                ui.checkbox(&mut self.legend, "")
                    .on_hover_text(ui.localize("legend.hover"));
                ui.end_row();
                // Link axis
                ui.label(ui.localize("link_axis"));
                ui.horizontal(|ui| {
                    ui.checkbox(&mut self.link.x, "")
                        .on_hover_text(ui.localize("link_axis.hover?axis=x"));
                    ui.checkbox(&mut self.link.y, "")
                        .on_hover_text(ui.localize("link_axis.hover?axis=y"));
                });
                ui.end_row();
                // Drag
                ui.label(ui.localize("drag"));
                ui.horizontal(|ui| {
                    ui.checkbox(&mut self.drag.x, "")
                        .on_hover_text(ui.localize("drag.hover?axis=x"));
                    ui.checkbox(&mut self.drag.y, "")
                        .on_hover_text(ui.localize("drag.hover?axis=y"));
                });
                ui.end_row();
                // Scroll
                ui.label(ui.localize("scroll"));
                ui.checkbox(&mut self.scroll, "")
                    .on_hover_text(ui.localize("scroll.hover"));
                ui.end_row();
                // Zoom
                ui.label(ui.localize("zoom"));
                ui.horizontal(|ui| {
                    ui.checkbox(&mut self.zoom.x, "")
                        .on_hover_text(ui.localize("zoom.hover?axis=x"));
                    ui.checkbox(&mut self.zoom.y, "")
                        .on_hover_text(ui.localize("zoom.hover?axis=y"));
                });
            });
        });
        ui.collapsing(RichText::new(ui.localize("source")).heading(), |ui| {
            Grid::new(ui.next_auto_id()).show(ui, |ui| {
                // Line
                ui.label(ui.localize("source_line"))
                    .on_hover_text(ui.localize("source_line.hover"));
                ui.checkbox(&mut self.source.line, "")
                    .on_hover_text(ui.localize("source_line.hover"));
                ui.end_row();
                // Points
                ui.label(ui.localize("source_points"))
                    .on_hover_text(ui.localize("source_points.hover"));
                ui.horizontal(|ui| {
                    ui.add(
                        DragValue::new(&mut self.source.points.radius)
                            .range(0.0..=f32::MAX)
                            .speed(0.1),
                    )
                    .on_hover_text(ui.localize("source_points_radius.hover"));
                    ui.checkbox(&mut self.source.points.filled, "")
                        .on_hover_text(ui.localize("source_points_fill.hover"));
                    ui.color_edit_button_srgba(&mut self.source.points.color)
                        .on_hover_text(ui.localize("source_points_color.hover"));
                });
            });
        });
        ui.collapsing(RichText::new(ui.localize("resampling")).heading(), |ui| {
            Grid::new(ui.next_auto_id()).show(ui, |ui| {
                //
                ui.label(ui.localize("resampling_by_mean"));
                ui.checkbox(&mut self.resampling.mean, "")
                    .on_hover_localized("resampling_by_mean.hover");
                ui.end_row();
                //
                ui.label(ui.localize("resampling_by_median"));
                ui.checkbox(&mut self.resampling.median, "")
                    .on_hover_localized("resampling_by_median.hover");
                if !self.resampling.mean && !self.resampling.median {
                    ui.disable();
                }
                ui.end_row();
                //
                ui.label(ui.localize("every"));
                ui.add(DragValue::new(&mut self.resampling.every).range(1..=86400))
                    .on_hover_localized("every.hover");
                ui.end_row();
                //
                ui.label(ui.localize("period"));
                ui.add(DragValue::new(&mut self.resampling.period).range(1..=86400))
                    .on_hover_localized("window_duration");
            });
        });
        ui.collapsing(RichText::new(ui.localize("rolling")).heading(), |ui| {
            ui.horizontal(|ui| {
                ui.label(ui.localize("mean"));
                ui.checkbox(&mut self.rolling.mean, "")
                    .on_hover_text(ui.localize("rolling_mean"));
            });
            ui.horizontal(|ui| {
                ui.label(ui.localize("median"));
                ui.checkbox(&mut self.rolling.median, "")
                    .on_hover_text(ui.localize("rolling_median"));
            });
            ui.horizontal(|ui| {
                ui.label(ui.localize("window_size"));
                ui.add(DragValue::new(&mut self.rolling.window_size).range(1..=usize::MAX))
                    .on_hover_text(ui.localize("window_size_description"));
            });
            ui.horizontal(|ui| {
                ui.label(ui.localize("min_periods"));
                ui.add(
                    DragValue::new(&mut self.rolling.min_periods)
                        .range(1..=self.rolling.window_size),
                )
                .on_hover_text(ui.localize("min_periods_description"));
            });
        });
    }
}

impl Default for Settings {
    fn default() -> Self {
        Self::new()
    }
}

impl Hash for Settings {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.drag.x.hash(state);
        self.drag.y.hash(state);
        self.legend.hash(state);
        self.link.x.hash(state);
        self.link.y.hash(state);
        self.scroll.hash(state);
        self.zoom.x.hash(state);
        self.zoom.y.hash(state);
        self.time.hash(state);
        self.source.hash(state);
        self.resampling.hash(state);
        self.rolling.hash(state);
    }
}

/// Source
#[derive(Clone, Copy, Debug, Deserialize, Hash, PartialEq, Serialize)]
pub(crate) struct Source {
    pub(crate) line: bool,
    pub(crate) points: Points,
}

impl Source {
    pub(crate) const fn new() -> Self {
        Self {
            line: true,
            points: Points {
                color: Color32::TRANSPARENT,
                filled: true,
                radius: 0.0,
            },
        }
    }
}

impl Default for Source {
    fn default() -> Self {
        Self::new()
    }
}

/// Downsampling
#[derive(Clone, Copy, Debug, Deserialize, Hash, PartialEq, Serialize)]
pub(crate) struct Resampling {
    pub(crate) mean: bool,
    pub(crate) median: bool,
    pub(crate) every: i64,
    pub(crate) period: i64,
}

impl Resampling {
    pub(crate) const fn new() -> Self {
        Self {
            mean: true,
            median: false,
            every: 60,
            period: 120,
        }
    }
}

impl Default for Resampling {
    fn default() -> Self {
        Self::new()
    }
}

/// Rolling
#[derive(Clone, Copy, Debug, Deserialize, Hash, PartialEq, Serialize)]
pub(crate) struct Rolling {
    pub(crate) mean: bool,
    pub(crate) median: bool,
    pub(crate) window_size: usize,
    pub(crate) min_periods: usize,
}

impl Rolling {
    pub(crate) const fn new() -> Self {
        Self {
            mean: false,
            median: false,
            window_size: 30,
            min_periods: 1,
        }
    }
}

impl Default for Rolling {
    fn default() -> Self {
        Self::new()
    }
}

// /// Concentration
// #[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
// pub(crate) struct Concentration {
//     pub(crate) unit: ConcentrationUnit,
// }

/// Points
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
pub(crate) struct Points {
    pub(crate) color: Color32,
    pub(crate) filled: bool,
    pub(crate) radius: f32,
}

impl Hash for Points {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.color.hash(state);
        self.filled.hash(state);
        self.radius.ord().hash(state);
    }
}

// /// Temperature
// #[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
// pub(crate) struct Temperature {
//     pub(crate) unit: TemperatureUnit,
// }

/// Time
#[derive(Clone, Copy, Debug, Deserialize, Hash, PartialEq, Serialize)]
pub(crate) struct Time {
    pub(crate) offset: UtcOffset,
    pub(crate) _offset: Offset,
}

impl Default for Time {
    fn default() -> Self {
        Self {
            offset: UtcOffset::UTC,
            _offset: Offset::Utc,
        }
    }
}

/// Offset
#[derive(Clone, Copy, Debug, Deserialize, Hash, PartialEq, Serialize)]
pub enum Offset {
    Utc,
    Local,
}

impl Offset {
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

    // fn time_zone(&self) -> Box<impl TimeZone> {
    //     match self {
    //         Self::Utc => Box::new(Utc),
    //         Self::Local => Box::new(Local),
    //     }
    // }
}

// impl Offset for UtcOffset {
//     fn name(&self) -> Cow<str> {
//         if *self == UtcOffset::UTC {
//             Cow::Borrowed("UTC")
//         } else {
//             Cow::Owned(self.to_string())
//         }
//     }
// }
