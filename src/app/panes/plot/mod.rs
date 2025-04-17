pub(crate) use self::settings::Settings;

use crate::{
    app::{
        DATE_TIME_FORMAT,
        computers::{PlotComputed, PlotKey},
        metadata::MetaDataFrame,
    },
    utils::hashed::Hashed,
};
use arrow::temporal_conversions::timestamp_ms_to_datetime;
use chrono::{
    DateTime, Datelike, Duration, DurationRound as _, FixedOffset, Local, NaiveDateTime,
    Offset as _, SubsecRound, TimeZone, Timelike, Utc,
};
use egui::{Id, Response, TextStyle, Ui, Widget, emath::round_to_decimals};
use egui_l20n::UiExt;
use egui_plot::{GridInput, GridMark, Legend, Line, Plot, PlotPoints, Points};
use settings::Offset;
use std::{
    fmt::Display,
    i16::MIN,
    iter::{self},
    ops::Range,
};
use tracing::trace;

use super::ID_SOURCE;

const SECOND: f64 = 1000.0;
const MINUTE: f64 = 60.0 * SECOND;
const HOUR: f64 = 60.0 * MINUTE;
const DAY: f64 = 24.0 * HOUR;

static YMDHMS: &str = "%Y-%m-%d %H:%M:%S";
static YMD: &str = "%Y-%m-%d";
static HMS: &str = "%H:%M:%S";
static MS: &str = "%M:%S";
static S: &str = "%S";

/// Plot view
#[derive(Debug, PartialEq)]
pub(crate) struct View<'a> {
    pub(crate) frame: &'a Hashed<MetaDataFrame>,
    pub(crate) settings: &'a mut Settings,
}

impl<'a> View<'a> {
    pub(crate) const fn new(frame: &'a Hashed<MetaDataFrame>, settings: &'a mut Settings) -> Self {
        Self { frame, settings }
    }
}

impl View<'_> {
    pub(crate) fn show(&mut self, ui: &mut Ui) {
        // Plot
        let mut plot = Plot::new(ID_SOURCE);
        if self.settings.legend {
            plot = plot.legend(Legend::default().text_style(TextStyle::Monospace));
        }
        plot.label_formatter(|name, value| {
            let mut formatted = String::new();
            if !name.is_empty() {
                formatted.push_str(&format!("{name}\n"));
            }
            let date_time = match self.settings.time._offset {
                Offset::Utc => format_time(value.x, Utc, YMDHMS),
                Offset::Local => format_time(value.x, Local, YMDHMS),
            };
            let value = round_to_decimals(value.y, 2);
            formatted.push_str(&format!("x = {date_time}\ny = {value}"));
            formatted
        })
        .allow_drag(self.settings.drag)
        .allow_scroll(self.settings.scroll)
        .allow_zoom(self.settings.zoom)
        .x_axis_label(ui.localize("time"))
        .x_axis_formatter(|grid_mark, _| match self.settings.time._offset {
            Offset::Utc => time_axis_formatter(grid_mark, Utc),
            Offset::Local => time_axis_formatter(grid_mark, Local),
        })
        .x_grid_spacer(|grid_input| time_grid_spacer(grid_input, Utc))
        // .y_axis_label(unit.abbreviation())
        // .y_axis_formatter(move |y, _| round_to_decimals(y.value, 2).to_string())
        .link_axis(Id::new("Plot"), self.settings.link)
        .link_cursor(Id::new("Plot"), self.settings.link)
        .show(ui, |ui| {
            let target = ui.ctx().memory_mut(|memory| {
                memory.caches.cache::<PlotComputed>().get(PlotKey {
                    frame: self.frame,
                    settings: self.settings,
                })
            });
            // Source
            if self.settings.source.line {
                for (identifier, points) in target.source {
                    // Line
                    ui.line(
                        Line::new(PlotPoints::new(points.clone())).name(format!("{identifier:x}")),
                    );
                    // Points
                    if self.settings.source.points.radius > 0.0 {
                        ui.points(
                            Points::new(PlotPoints::new(points))
                                .color(self.settings.source.points.color)
                                .filled(self.settings.source.points.filled)
                                .radius(self.settings.source.points.radius)
                                .name(format!("{identifier:x}")),
                        );
                    }
                }
            }
            // Resampling mean
            if self.settings.resampling.mean {
                for (identifier, points) in target.resampling.mean {
                    let line = Line::new(PlotPoints::new(points)).name(format!("{identifier:x}"));
                    ui.line(line);
                }
            }
            // Resampling median
            if self.settings.resampling.median {
                for (identifier, points) in target.resampling.median {
                    let line = Line::new(PlotPoints::new(points)).name(format!("{identifier:x}"));
                    ui.line(line);
                }
            }
            // Rolling mean
            if self.settings.rolling.mean {
                for (identifier, points) in target.rolling.mean {
                    let line = Line::new(PlotPoints::new(points)).name(format!("{identifier:x}"));
                    ui.line(line);
                }
            }
            // Rolling median
            if self.settings.rolling.median {
                for (identifier, points) in target.rolling.median {
                    let line = Line::new(PlotPoints::new(points)).name(format!("{identifier:x}"));
                    ui.line(line);
                }
            }
        });
    }
}

// impl Widget for View<'_> {
//     fn ui(self, ui: &mut Ui) -> Response {
//         // Plot
//         let mut plot = Plot::new("plot");
//         if self.settings.legend {
//             plot = plot.legend(Legend::default());
//         }
//         plot.label_formatter(|name, value| {
//             let mut formatted = String::new();
//             if !name.is_empty() {
//                 formatted.push_str(&format!("File: {name}\n"));
//             }
//             let time = format_time(value.x, self.settings.time.offset, YMDHMS);
//             let temperature = round_to_decimals(value.y, 2);
//             formatted.push_str(&format!("x = {time}\ny = {temperature}"));
//             formatted
//         })
//         .allow_drag(self.settings.drag)
//         .allow_scroll(self.settings.scroll)
//         .allow_zoom(self.settings.zoom)
//         // .x_axis_label("Time")
//         .x_axis_formatter(|grid_mark, _| time_axis_formatter(grid_mark, self.settings.time.offset))
//         .x_grid_spacer(|grid_input| time_grid_spacer(grid_input, self.settings.time.offset))
//         // .y_axis_label(unit.abbreviation())
//         // .y_axis_formatter(move |y, _| round_to_decimals(y.value, 2).to_string())
//         .link_axis(Id::new("plot"), self.settings.link)
//         .link_cursor(Id::new("plot"), self.settings.link)
//         .show(ui, |ui| {
//             let computed = ui.ctx().memory_mut(|memory| {
//                 memory.caches.cache::<PlotComputed>().get(PlotKey {
//                     frame: self.frame,
//                     settings: self.settings,
//                 })
//             });
//             // Source
//             // Line
//             ui.line(Line::new(PlotPoints::new(computed.source.clone())).name("Source line"));
//             // Points
//             if self.settings.source.points.radius > 0.0 {
//                 ui.points(
//                     Points::new(computed.source)
//                         .color(self.settings.source.points.color)
//                         .filled(self.settings.source.points.filled)
//                         .radius(self.settings.source.points.radius)
//                         .name("Source points"),
//                 );
//             }
//             // Resampling
//             if let Some(points) = computed.resampling.mean {
//                 ui.line(Line::new(PlotPoints::new(points)).name(format!("Resampling mean")));
//             }
//             if let Some(points) = computed.resampling.median {
//                 ui.line(Line::new(PlotPoints::new(points)).name(format!("Resampling median")));
//             }
//             // Rolling
//             if let Some(points) = computed.rolling.mean {
//                 ui.line(Line::new(PlotPoints::new(points)).name(format!("Rolling mean")));
//             }
//             if let Some(points) = computed.rolling.median {
//                 ui.line(Line::new(PlotPoints::new(points)).name(format!("Rolling median")));
//             }
//             // // Resampling
//             // if self.settings.resampling.enable {
//             //     let data_frame = ui.ctx().memory_mut(|memory| {
//             //         memory.caches.cache::<Resampled>().get(ResamplerKey {
//             //             data_frame: &data_frame,
//             //             resampling: &self.settings.resampling,
//             //         })
//             //     });
//             //     let points = points(&data_frame, name)?;
//             //     let line = Line::new(PlotPoints::new(points.clone())).name(format!("Resampling"));
//             //     ui.line(line);
//             // }
//             // // Rolling
//             // if self.settings.rolling.mean || self.settings.rolling.median {
//             //     let data_frame = ui.ctx().memory_mut(|memory| {
//             //         memory.caches.cache::<Rolled>().get(RollerKey {
//             //             data_frame: &data_frame,
//             //             rolling: &self.settings.rolling,
//             //         })
//             //     });
//             //     // Mean
//             //     if self.settings.rolling.mean {
//             //         let points = points(&data_frame, &format!("{name}.RollingMean"))?;
//             //         let line = Line::new(PlotPoints::new(points)).name("Rolling mean");
//             //         ui.line(line);
//             //         // .filter_map(|(milliseconds, value)| {
//             //         //     Some([(milliseconds? as f64 / 1000f64) as _, value?])
//             //         // })
//             //     }
//             //     // Median
//             //     if self.settings.rolling.median {
//             //         let points = points(&data_frame, &format!("{name}.RollingMedian"))?;
//             //         let line = Line::new(PlotPoints::new(points)).name("Rolling median");
//             //         ui.line(line);
//             //     }
//             // }
//             Ok::<_, PolarsError>(())
//         })
//         .response
//     }
// }

fn time_grid_spacer<T: TimeZone>(grid_input: GridInput, time_zone: T) -> Vec<GridMark> {
    let mut marks = vec![];
    let (min, max) = grid_input.bounds;
    let range = max - min;
    let mut min = date_time(min, &time_zone);
    let mut max = date_time(max, &time_zone);
    let step = if range > 31.0 * DAY {
        return marks;
    } else if range > DAY {
        const HOUR: Duration = Duration::minutes(1);
        min = min.clone().duration_round(HOUR).unwrap_or(min);
        max = max.clone().duration_round_up(HOUR).unwrap_or(max);
        HOUR
    } else if range > HOUR {
        const MINUTE: Duration = Duration::minutes(1);
        min = min.clone().duration_round(MINUTE).unwrap_or(min);
        max = max.clone().duration_round_up(MINUTE).unwrap_or(max);
        MINUTE
    } else {
        min = min.trunc_subsecs(0);
        max = max.round_subsecs(0);
        Duration::seconds(1)
    };
    trace!(%range, %step);
    let mut date_time = min;
    while date_time <= max {
        if date_time.second() == 0 {
            if date_time.minute() == 0 {
                if date_time.hour() == 0 {
                    marks.push(GridMark {
                        value: date_time.timestamp_millis() as f64,
                        step_size: DAY,
                    });
                } else {
                    marks.push(GridMark {
                        value: date_time.timestamp_millis() as f64,
                        step_size: HOUR,
                    });
                }
            } else {
                marks.push(GridMark {
                    value: date_time.timestamp_millis() as f64,
                    step_size: MINUTE,
                });
            }
        } else {
            marks.push(GridMark {
                value: date_time.timestamp_millis() as f64,
                step_size: SECOND,
            });
        }
        date_time += step;
    }
    marks
}

fn time_axis_formatter<T: TimeZone<Offset: Display>>(grid_mark: GridMark, time_zone: T) -> String {
    match grid_mark.step_size {
        SECOND => format_time(grid_mark.value, time_zone, S),
        MINUTE => format_time(grid_mark.value, time_zone, MS),
        HOUR => format_time(grid_mark.value, time_zone, HMS),
        DAY => format_time(grid_mark.value, time_zone, YMD),
        _ => String::new(),
    }
}

fn format_time<T: TimeZone<Offset: Display>>(value: f64, time_zone: T, format: &str) -> String {
    date_time(value, &time_zone).format(format).to_string()
}

fn date_time<T: TimeZone>(value: f64, time_zone: &T) -> DateTime<T> {
    timestamp_ms_to_datetime(value as _)
        .map(|date_time| time_zone.from_utc_datetime(&date_time))
        .unwrap_or_else(|| Utc::now().with_timezone(time_zone))
}

mod settings;
