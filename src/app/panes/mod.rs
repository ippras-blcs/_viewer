use self::{
    plot::{Settings as PlotSettings, View as PlotView},
    state::State,
    table::{Settings as TableSettings, View as TableView},
    view::{View, ViewWidget},
};
use super::metadata::{ICON, MAX_TIMESTAMP, MIN_TIMESTAMP, NAME};
use crate::{
    app::{
        computers::{TableComputed, TableKey},
        metadata::{MetaDataFrame, Metadata},
        mqtt::{
            TOPIC_ATUC, TOPIC_DDOC_C1, TOPIC_DDOC_C2, TOPIC_DDOC_T1, TOPIC_DDOC_T2, TOPIC_DDOC_V1,
            TOPIC_DDOC_V2, TOPIC_DTEC,
        },
    },
    utils::hashed::Hashed,
};
use egui::{CursorIcon, Response, RichText, Ui, Window, util::hash};
use egui_l20n::{ResponseExt, UiExt as _};
use egui_phosphor::regular::{ARROWS_CLOCKWISE, ARROWS_HORIZONTAL, GEAR, MINUS};
use polars::prelude::*;
use serde::{Deserialize, Serialize};

const ID_SOURCE: &str = "Pane";

/// Pane
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub(crate) struct Pane {
    pub(crate) kind: Kind,
    pub(crate) source: Hashed<MetaDataFrame>,
    pub(crate) target: Hashed<DataFrame>,
    pub(crate) table_settings: TableSettings,
    pub(crate) plot_settings: PlotSettings,
    pub(crate) state: State,
    pub(crate) view: View,
}

impl Pane {
    pub(crate) const ATUC: Self = Self::new(Kind::Atuc);
    pub(crate) const DDOC_C1: Self = Self::new(Kind::Ddoc(Ddoc::C1));
    pub(crate) const DDOC_C2: Self = Self::new(Kind::Ddoc(Ddoc::C2));
    pub(crate) const DDOC_T1: Self = Self::new(Kind::Ddoc(Ddoc::T1));
    pub(crate) const DDOC_T2: Self = Self::new(Kind::Ddoc(Ddoc::T2));
    pub(crate) const DDOC_V1: Self = Self::new(Kind::Ddoc(Ddoc::V1));
    pub(crate) const DDOC_V2: Self = Self::new(Kind::Ddoc(Ddoc::V2));
    pub(crate) const DTEC: Self = Self::new(Kind::Dtec);
}

impl Pane {
    pub(crate) const fn new(kind: Kind) -> Self {
        Self {
            kind,
            source: Hashed {
                value: MetaDataFrame::new(Metadata::new(), DataFrame::empty()),
                hash: 0,
            },
            target: Hashed {
                value: DataFrame::empty(),
                hash: 0,
            },
            table_settings: TableSettings::new(),
            plot_settings: PlotSettings::new(),
            state: State::new(),
            view: View::Table,
        }
    }

    pub(crate) fn icon(&self) -> &str {
        &self.source.meta[ICON]
    }

    pub(crate) fn title(&self) -> String {
        format!(
            "{} {MINUS} {}",
            self.source.meta[MIN_TIMESTAMP], self.source.meta[MAX_TIMESTAMP],
        )
    }

    pub(crate) fn name(&self) -> &str {
        &self.source.meta[NAME]
    }

    pub(crate) const fn topic(&self) -> Option<&str> {
        if self.is_real_time() {
            Some(self.kind.topic())
        } else {
            None
        }
    }

    pub(crate) const fn is_real_time(&self) -> bool {
        // self.source.is_none()
        false
    }

    pub(crate) fn text(&self) -> &'static str {
        match self.kind {
            Kind::Atuc => "analog_turbidity_controller.abbreviation",
            Kind::Ddoc(Ddoc::C1) => {
                "digital_disolved_oxygen_controller_concentration_channel?index=1"
            }
            Kind::Ddoc(Ddoc::C2) => {
                "digital_disolved_oxygen_controller_concentration_channel?index=2"
            }
            Kind::Ddoc(Ddoc::T1) => {
                "digital_disolved_oxygen_controller_temperature_channel?index=1"
            }
            Kind::Ddoc(Ddoc::T2) => {
                "digital_disolved_oxygen_controller_temperature_channel?index=2"
            }
            Kind::Ddoc(Ddoc::V1) => "digital_disolved_oxygen_controller_voltage_channel?index=1",
            Kind::Ddoc(Ddoc::V2) => "digital_disolved_oxygen_controller_voltage_channel?index=2",
            Kind::Dtec => "digital_temperature_controller.abbreviation",
        }
    }

    pub(crate) fn hover_text(&self) -> &'static str {
        match self.kind {
            Kind::Atuc => "analog_turbidity_controller.hover",
            Kind::Ddoc(Ddoc::C1) => {
                "digital_disolved_oxygen_controller_concentration_channel?index=1"
            }
            Kind::Ddoc(Ddoc::C2) => {
                "digital_disolved_oxygen_controller_concentration_channel?index=2"
            }
            Kind::Ddoc(Ddoc::T1) => {
                "digital_disolved_oxygen_controller_temperature_channel?index=1"
            }
            Kind::Ddoc(Ddoc::T2) => {
                "digital_disolved_oxygen_controller_temperature_channel?index=2"
            }
            Kind::Ddoc(Ddoc::V1) => "digital_disolved_oxygen_controller_voltage_channel?index=1",
            Kind::Ddoc(Ddoc::V2) => "digital_disolved_oxygen_controller_voltage_channel?index=2",
            Kind::Dtec => "digital_temperature_controller.hover",
        }
    }
}

impl Pane {
    pub(crate) fn header(&mut self, ui: &mut Ui) -> Response {
        let mut response = ui.heading(self.icon()).on_hover_localized(self.name());
        response |= ui.heading(self.title());
        response = response
            .on_hover_text(format!("{:x}/{:x}", self.source.hash, self.target.hash))
            .on_hover_cursor(CursorIcon::Grab);
        ui.separator();
        // View
        ui.add(ViewWidget::new(&mut self.view));
        ui.separator();

        // Reset
        if ui
            .button(RichText::new(ARROWS_CLOCKWISE).heading())
            .on_hover_ui(|ui| {
                ui.label(ui.localize("reset_table"));
            })
            .clicked()
        {
            self.state.reset_table_state = true;
        }
        // Resize
        ui.toggle_value(
            &mut self.table_settings.resizable,
            RichText::new(ARROWS_HORIZONTAL).heading(),
        )
        .on_hover_ui(|ui| {
            ui.label(ui.localize("resize_table"));
        });
        ui.separator();
        // Settings
        ui.toggle_value(
            &mut self.state.open_settings_window,
            RichText::new(GEAR).heading(),
        )
        .on_hover_ui(|ui| {
            ui.label(ui.localize("settings"));
        });

        ui.separator();
        response
    }

    // https://github.com/rerun-io/egui_tiles/blob/1be4183f7c76cc96cadd8b0367f84c48a8e1b4bd/src/container/tabs.rs#L57
    // https://github.com/emilk/egui/discussions/3468
    pub(crate) fn body(&mut self, ui: &mut Ui) {
        // let Some(ref data_frame) = self.data_frame.clone().or_else(|| {
        //     let topic = self.topic()?;
        //     let store = ui.data(|data| data.get_temp::<Arc<InMemory>>(Id::new(topic)))?;
        //     let path = Path::from(topic.to_owned());
        //     tokio::spawn(async move {
        //         let result = store.get(&path).await?;
        //         let bytes = result.bytes().await?;
        //         let mut reader = ParquetReader::new(Cursor::new(bytes));
        //         let meta = reader.get_metadata()?;
        //         let data = reader.finish()?;
        //         print!("data: {data:?}");
        //         Ok::<_, Error>(())
        //     });
        //     // panic!("!!!");
        //     // Some(())
        // }) else {
        //     ui.centered_and_justified(|ui| ui.spinner());
        //     return;
        // };

        // ui.centered_and_justified(|ui| ui.spinner());
        // return;
        self.windows(ui);
        match self.view {
            View::Plot => {
                PlotView::new(&self.source, &mut self.plot_settings).show(ui);
            }
            View::Table => {
                self.target = ui.memory_mut(|memory| {
                    let key = TableKey {
                        frame: &self.source,
                        settings: &self.table_settings,
                    };
                    Hashed {
                        value: memory.caches.cache::<TableComputed>().get(key),
                        hash: hash(key),
                    }
                });
                TableView::new(&self.target, &self.table_settings, &mut self.state).show(ui);
            }
        }
    }

    fn windows(&mut self, ui: &mut Ui) {
        // Settings
        let mut open_settings_window = self.state.open_settings_window;
        Window::new(format!("{GEAR} Pane settings"))
            .id(ui.auto_id_with(ID_SOURCE))
            .default_pos(ui.next_widget_position())
            .open(&mut open_settings_window)
            .show(ui.ctx(), |ui| match self.view {
                View::Plot => self.plot_settings.show(ui),
                View::Table => self.table_settings.show(ui, &self.source.data.clone()),
            });
        self.state.open_settings_window = open_settings_window;
    }
}

/// Kind
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
pub(crate) enum Kind {
    Atuc,
    Ddoc(Ddoc),
    Dtec,
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
            Kind::Atuc => TOPIC_ATUC,
            Kind::Ddoc(Ddoc::C1) => TOPIC_DDOC_C1,
            Kind::Ddoc(Ddoc::C2) => TOPIC_DDOC_C2,
            Kind::Ddoc(Ddoc::T1) => TOPIC_DDOC_T1,
            Kind::Ddoc(Ddoc::T2) => TOPIC_DDOC_T2,
            Kind::Ddoc(Ddoc::V1) => TOPIC_DDOC_V1,
            Kind::Ddoc(Ddoc::V2) => TOPIC_DDOC_V2,
            Kind::Dtec => TOPIC_DTEC,
        }
    }
}

pub(crate) mod behavior;
pub(crate) mod plot;
pub(crate) mod settings;
pub(crate) mod state;
pub(crate) mod table;
pub(crate) mod view;
