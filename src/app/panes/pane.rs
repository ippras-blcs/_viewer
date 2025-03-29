use super::{plot::Pane as PlotPane, settings::Settings, table::Pane as TablePane};
use crate::{
    app::{
        MQTT_TOPIC_DDOC_C1, MQTT_TOPIC_DDOC_C2, MQTT_TOPIC_DDOC_T1, MQTT_TOPIC_DDOC_T2,
        MQTT_TOPIC_DDOC_V1, MQTT_TOPIC_DDOC_V2, MQTT_TOPIC_TEMPERATURE, MQTT_TOPIC_TURBIDITY,
        NAME_DDOC_C1, NAME_DDOC_C2, NAME_DDOC_T1, NAME_DDOC_T2, NAME_DDOC_V1, NAME_DDOC_V2,
        NAME_TEMPERATURE, NAME_TURBIDITY,
    },
    localization::titlecase,
};
use anyhow::Result;
use egui::{menu::bar, Id, Ui};
use egui_phosphor::regular::{CHART_LINE, CLOCK, CLOUD, TABLE};
use polars::prelude::*;
use serde::{Deserialize, Serialize};
use std::fmt::Write;
use time::OffsetDateTime;
use tracing::error;

/// Pane
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub(crate) struct Pane {
    pub(crate) kind: Kind,
    pub(crate) data_frame: Option<DataFrame>,
    pub(crate) settings: Settings,
    pub(crate) view: View,
}

impl Pane {
    pub(crate) const TEMPERATURE: Self = Self::new(Kind::Temperature);
    pub(crate) const TURBIDITY: Self = Self::new(Kind::Turbidity);
    pub(crate) const DDOC_C1: Self = Self::new(Kind::Ddoc(Ddoc::C1));
    pub(crate) const DDOC_C2: Self = Self::new(Kind::Ddoc(Ddoc::C2));
    pub(crate) const DDOC_T1: Self = Self::new(Kind::Ddoc(Ddoc::T1));
    pub(crate) const DDOC_T2: Self = Self::new(Kind::Ddoc(Ddoc::T2));
    pub(crate) const DDOC_V1: Self = Self::new(Kind::Ddoc(Ddoc::V1));
    pub(crate) const DDOC_V2: Self = Self::new(Kind::Ddoc(Ddoc::V2));
}

impl Pane {
    pub(crate) const fn new(kind: Kind) -> Self {
        Self {
            kind,
            data_frame: None,
            settings: Settings::new(),
            view: View::Plot,
        }
    }

    pub(crate) const fn icon(&self) -> &str {
        if self.is_real_time() {
            CLOCK
        } else {
            CLOUD
        }
    }

    pub(crate) const fn name(&self) -> &str {
        match self.kind {
            Kind::Temperature => NAME_TEMPERATURE,
            Kind::Turbidity => NAME_TURBIDITY,
            Kind::Ddoc(Ddoc::C1) => NAME_DDOC_C1,
            Kind::Ddoc(Ddoc::C2) => NAME_DDOC_C2,
            Kind::Ddoc(Ddoc::T1) => NAME_DDOC_T1,
            Kind::Ddoc(Ddoc::T2) => NAME_DDOC_T2,
            Kind::Ddoc(Ddoc::V1) => NAME_DDOC_V1,
            Kind::Ddoc(Ddoc::V2) => NAME_DDOC_V2,
        }
    }

    pub(crate) const fn topic(&self) -> Option<&str> {
        if self.is_real_time() {
            Some(self.kind.topic())
        } else {
            None
        }
    }

    pub(crate) fn title(&self) -> String {
        let mut title = format!("{} {}", self.icon(), self.name());
        if let Err(error) = (|| -> Result<_> {
            if let Some(data_frame) = &self.data_frame {
                if let Some(date) = data_frame["Time"].datetime()?.to_string("%Y-%m-%d")?.get(0) {
                    write!(&mut title, " {date}")?;
                }
            } else {
                let date = OffsetDateTime::now_local()?.date();
                write!(&mut title, " {date}")?;
            }
            Ok(())
        })() {
            error!(%error);
        }
        title
    }

    pub(crate) const fn is_real_time(&self) -> bool {
        self.data_frame.is_none()
    }

    pub(crate) fn text(&self) -> String {
        match self.kind {
            Kind::Temperature => titlecase!("temperature"),
            Kind::Turbidity => titlecase!("turbidity"),
            Kind::Ddoc(Ddoc::C1) => titlecase!("ddoc_c1"),
            Kind::Ddoc(Ddoc::C2) => titlecase!("ddoc_c2"),
            Kind::Ddoc(Ddoc::T1) => titlecase!("temperature_channel?index=1"),
            Kind::Ddoc(Ddoc::T2) => titlecase!("temperature_channel?index=2"),
            Kind::Ddoc(Ddoc::V1) => titlecase!("ddoc_v1"),
            Kind::Ddoc(Ddoc::V2) => titlecase!("ddoc_v2"),
        }
    }

    pub(crate) fn hover_text(&self) -> String {
        match self.kind {
            Kind::Temperature => titlecase!("temperature"),
            Kind::Turbidity => titlecase!("turbidity"),
            Kind::Ddoc(Ddoc::C1) => titlecase!("ddoc_c1"),
            Kind::Ddoc(Ddoc::C2) => titlecase!("ddoc_c2"),
            Kind::Ddoc(Ddoc::T1) => {
                titlecase!("digital_disolved_oxygen_controller.temperature_channel?index=1")
            }
            Kind::Ddoc(Ddoc::T2) => {
                titlecase!("digital_disolved_oxygen_controller.temperature_channel?index=2")
            }
            Kind::Ddoc(Ddoc::V1) => {
                titlecase!("digital_disolved_oxygen_controller.concentration_channel?index=1")
            }
            Kind::Ddoc(Ddoc::V2) => {
                titlecase!("digital_disolved_oxygen_controller.concentration_channel?index=2")
            }
        }
    }
}

impl Pane {
    // https://github.com/rerun-io/egui_tiles/blob/1be4183f7c76cc96cadd8b0367f84c48a8e1b4bd/src/container/tabs.rs#L57
    // https://github.com/emilk/egui/discussions/3468
    pub(crate) fn ui(&mut self, ui: &mut Ui) {
        let Some(ref data_frame) = self
            .data_frame
            .clone()
            .or_else(|| ui.data(|data| data.get_temp::<DataFrame>(Id::new(self.topic()?))))
        else {
            ui.centered_and_justified(|ui| ui.spinner());
            return;
        };
        match self.view {
            View::Plot => ui.add(PlotPane {
                data_frame,
                settings: &mut self.settings,
            }),
            View::Table => ui.add(TablePane {
                data_frame,
                settings: &mut self.settings,
            }),
        };
    }

    pub(crate) fn settings(&mut self, ui: &mut Ui) {
        bar(ui, |ui| {
            ui.selectable_value(&mut self.view, View::Plot, View::Plot.icon());
            ui.selectable_value(&mut self.view, View::Table, View::Table.icon())
                .on_hover_text(titlecase!("pane_view"));
        });
        self.settings.ui(ui)
    }
}

/// Kind
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
pub(crate) enum Kind {
    Temperature,
    Turbidity,
    Ddoc(Ddoc),
}

/// Digital Disolved Oxygen Controller
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
pub(crate) enum Ddoc {
    C1,
    C2,
    T1,
    T2,
    V1,
    V2,
}

impl Kind {
    pub(crate) const fn topic(&self) -> &str {
        match self {
            Kind::Temperature => MQTT_TOPIC_TEMPERATURE,
            Kind::Turbidity => MQTT_TOPIC_TURBIDITY,
            Kind::Ddoc(Ddoc::C1) => MQTT_TOPIC_DDOC_C1,
            Kind::Ddoc(Ddoc::C2) => MQTT_TOPIC_DDOC_C2,
            Kind::Ddoc(Ddoc::T1) => MQTT_TOPIC_DDOC_T1,
            Kind::Ddoc(Ddoc::T2) => MQTT_TOPIC_DDOC_T2,
            Kind::Ddoc(Ddoc::V1) => MQTT_TOPIC_DDOC_V1,
            Kind::Ddoc(Ddoc::V2) => MQTT_TOPIC_DDOC_V2,
        }
    }
}

/// View
#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Serialize)]
pub(crate) enum View {
    #[default]
    Plot,
    Table,
}

impl View {
    pub(crate) const fn icon(&self) -> &str {
        match self {
            Self::Plot => CHART_LINE,
            Self::Table => TABLE,
        }
    }
}
