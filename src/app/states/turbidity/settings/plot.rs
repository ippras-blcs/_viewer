use egui::{Color32, DragValue, Grid, RichText, Ui, Vec2b, emath::Float};
use egui_l20n::{ResponseExt, UiExt};
use serde::{Deserialize, Serialize};
use std::hash::{Hash, Hasher};

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
            source: Source::new(),
            resampling: Resampling::new(),
            rolling: Rolling::new(),
        }
    }
}

impl Settings {
    pub(crate) fn show(&mut self, ui: &mut Ui) {
        ui.collapsing(RichText::new(ui.localize("plot")).heading(), |ui| {
            Grid::new(ui.next_auto_id()).show(ui, |ui| {
                // Legend
                ui.label(ui.localize("plot__legend"));
                ui.checkbox(&mut self.legend, "")
                    .on_hover_localized("plot__legend.hover");
                ui.end_row();
                // Link axis
                ui.label(ui.localize("plot__link_axis"));
                ui.horizontal(|ui| {
                    ui.checkbox(&mut self.link.x, "")
                        .on_hover_localized("plot__link_axis.hover?axis=x");
                    ui.checkbox(&mut self.link.y, "")
                        .on_hover_localized("plot__link_axis.hover?axis=y");
                });
                ui.end_row();
                // Drag
                ui.label(ui.localize("plot__drag"));
                ui.horizontal(|ui| {
                    ui.checkbox(&mut self.drag.x, "")
                        .on_hover_localized("plot__drag.hover?axis=x");
                    ui.checkbox(&mut self.drag.y, "")
                        .on_hover_localized("plot__drag.hover?axis=y");
                });
                ui.end_row();
                // Scroll
                ui.label(ui.localize("plot__scroll"));
                ui.checkbox(&mut self.scroll, "")
                    .on_hover_localized("plot__scroll.hover");
                ui.end_row();
                // Zoom
                ui.label(ui.localize("plot__zoom"));
                ui.horizontal(|ui| {
                    ui.checkbox(&mut self.zoom.x, "")
                        .on_hover_localized("plot__zoom.hover?axis=x");
                    ui.checkbox(&mut self.zoom.y, "")
                        .on_hover_localized("plot__zoom.hover?axis=y");
                });
            });
        });
        ui.collapsing(RichText::new(ui.localize("source")).heading(), |ui| {
            Grid::new(ui.next_auto_id()).show(ui, |ui| {
                // Line
                ui.label(ui.localize("source__line"))
                    .on_hover_localized("source__line.hover");
                ui.checkbox(&mut self.source.line, "")
                    .on_hover_localized("source__line.hover");
                ui.end_row();
                // Points
                ui.label(ui.localize("source__points"))
                    .on_hover_localized("source__points.hover");
                ui.horizontal(|ui| {
                    ui.add(
                        DragValue::new(&mut self.source.points.radius)
                            .range(0.0..=f32::MAX)
                            .speed(0.1),
                    )
                    .on_hover_localized("source__points_radius.hover");
                    ui.checkbox(&mut self.source.points.filled, "")
                        .on_hover_localized("source__points_fill.hover");
                    ui.color_edit_button_srgba(&mut self.source.points.color)
                        .on_hover_localized("source__points_color.hover");
                });
            });
        });
        ui.collapsing(RichText::new(ui.localize("resampling")).heading(), |ui| {
            Grid::new(ui.next_auto_id()).show(ui, |ui| {
                // Mean
                ui.label(ui.localize("resampling__mean"));
                ui.checkbox(&mut self.resampling.mean, "")
                    .on_hover_localized("resampling__mean.hover");
                ui.end_row();
                // Median
                ui.label(ui.localize("resampling__median"));
                ui.checkbox(&mut self.resampling.median, "")
                    .on_hover_localized("resampling__median.hover");
                ui.end_row();
                if !self.resampling.mean && !self.resampling.median {
                    ui.disable();
                }
                // Every
                ui.label(ui.localize("resampling__every"));
                ui.add(DragValue::new(&mut self.resampling.every).range(1..=86400))
                    .on_hover_localized("resampling__every.hover");
                ui.end_row();
                // Period
                ui.label(ui.localize("resampling__period"));
                ui.add(DragValue::new(&mut self.resampling.period).range(1..=86400))
                    .on_hover_localized("resampling__period.hover");
            });
        });
        ui.collapsing(RichText::new(ui.localize("rolling")).heading(), |ui| {
            Grid::new(ui.next_auto_id()).show(ui, |ui| {
                // Mean
                ui.label(ui.localize("rolling__mean"));
                ui.checkbox(&mut self.rolling.mean, "")
                    .on_hover_localized("rolling__mean.hover");
                ui.end_row();
                // Median
                ui.label(ui.localize("rolling__median"));
                ui.checkbox(&mut self.rolling.median, "")
                    .on_hover_localized("rolling__median.hover");
                ui.end_row();
                if !self.rolling.mean && !self.rolling.median {
                    ui.disable();
                }
                // Window size
                ui.label(ui.localize("rolling__window_size"));
                ui.add(DragValue::new(&mut self.rolling.window_size).range(1..=usize::MAX))
                    .on_hover_localized("rolling__window_size.hover");
                ui.end_row();
                // Min periods
                ui.label(ui.localize("rolling__min_periods"));
                ui.add(
                    DragValue::new(&mut self.rolling.min_periods)
                        .range(1..=self.rolling.window_size),
                )
                .on_hover_localized("rolling__min_periods.hover");
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
