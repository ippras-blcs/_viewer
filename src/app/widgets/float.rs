use egui::{DragValue, InnerResponse, Label, RichText, Ui, Widget, vec2};
use polars::prelude::*;

/// Float widget
pub(crate) struct FloatWidget {
    value: Option<f64>,
    settings: Settings,
}

impl FloatWidget {
    pub(crate) fn new(value: Option<f64>) -> Self {
        Self {
            value,
            settings: Settings::default(),
        }
    }

    pub(crate) fn disable(mut self, disable: bool) -> Self {
        self.settings.disable = disable;
        self
    }

    pub(crate) fn editable(mut self, editable: bool) -> Self {
        self.settings.editable = editable;
        self
    }

    pub(crate) fn hover(mut self, hover: bool) -> Self {
        self.settings.hover = hover;
        self
    }

    pub(crate) fn percent(mut self, percent: bool) -> Self {
        self.settings.percent = percent;
        self
    }

    pub(crate) fn precision(mut self, precision: Option<usize>) -> Self {
        self.settings.precision = precision;
        self
    }

    pub(crate) fn show(self, ui: &mut Ui) -> InnerResponse<Option<Option<f64>>> {
        let format = |value: f64| match self.settings.precision {
            Some(precision) => format!("{value:.precision$}"),
            None => AnyValue::from(value).to_string(),
        };
        if self.settings.disable {
            ui.disable();
        }
        let mut inner = None;
        let Some(mut value) = self.value else {
            // None
            let response = ui.add_sized(
                vec2(ui.available_width(), ui.spacing().interact_size.y),
                Label::new("None"),
            );
            if self.settings.editable {
                response.context_menu(|ui| {
                    if ui.button("Some").clicked() {
                        inner = Some(Some(0.0));
                    }
                });
            }
            return InnerResponse::new(inner, response);
        };
        // Percent
        if self.settings.percent {
            value *= 100.0;
        }
        // Editable
        let mut response = if self.settings.editable {
            // Writable
            let response = ui.add_sized(
                vec2(ui.available_width(), ui.spacing().interact_size.y),
                DragValue::new(&mut value)
                    .range(0.0..=f64::MAX)
                    .custom_formatter(|value, _| format(value)),
            );
            if response.changed() {
                inner = Some(Some(value));
            }
            let mut none = false;
            response.context_menu(|ui| {
                none = ui.button("None").clicked();
            });
            if none {
                inner = Some(None);
            }
            response
        } else {
            // Readable
            ui.label(format(value))
        };
        if self.settings.hover {
            response = response.on_hover_text(RichText::new(AnyValue::Float64(value).to_string()));
        }
        InnerResponse::new(inner, response)
    }
}

impl Widget for FloatWidget {
    fn ui(self, ui: &mut Ui) -> egui::Response {
        self.show(ui).response
    }
}

/// Settings
#[derive(Clone, Copy, Debug, Default)]
struct Settings {
    editable: bool,
    disable: bool,
    percent: bool,
    hover: bool,
    precision: Option<usize>,
}
