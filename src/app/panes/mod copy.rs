use self::{
    plot::View as PlotView,
    table::View as TableView,
    view::{View, ViewWidget},
};
use crate::{
    app::{
        YMDHMS,
        computers::table::{Computed as TableComputed, Key as TableKey},
        mqtt::{
            TOPIC_ATUC, TOPIC_DDOC_C1, TOPIC_DDOC_C2, TOPIC_DDOC_T1, TOPIC_DDOC_T2, TOPIC_DDOC_V1,
            TOPIC_DDOC_V2, TOPIC_DTEC,
        },
        panes::behavior::Behavior,
        states::pane::State,
    },
    export::xlsx,
    utils::hashed::{HashedDataFrame, HashedMetaDataFrame},
};
use anyhow::Result;
use chrono::NaiveDateTime;
use egui::{
    CentralPanel, CursorIcon, Frame, Id, MenuBar, Response, RichText, ScrollArea, TextStyle,
    TopBottomPanel, Ui, Window, util::hash,
};
use egui_l20n::{ResponseExt, UiExt as _};
use egui_phosphor::regular::{ARROWS_CLOCKWISE, ARROWS_HORIZONTAL, FLOPPY_DISK, GEAR, MINUS, X};
use egui_tiles::{TileId, UiResponse};
use metadata::{Metadata, NAME};
use polars::prelude::*;
use serde::{Deserialize, Serialize};
use tracing::instrument;

const ID_SOURCE: &str = "Pane";

/// Pane
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub(crate) struct Pane {
    id: Option<Id>,
    pub(crate) kind: Kind,
    pub(crate) frame: HashedMetaDataFrame,
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
            frame: HashedMetaDataFrame::new(Metadata::new(), HashedDataFrame::EMPTY),
            view: View::Table,
        }
    }

    pub(crate) fn icon(&self) -> &str {
        &self.frame.meta[ICON]
    }

    pub(crate) fn title(&self) -> String {
        // let min_timestamp =
        //     NaiveDateTime::parse_from_str(&self.source.meta[MIN_TIMESTAMP], DATE_TIME_FORMAT)
        //         .unwrap()
        //         .and_local_timezone(self.settings.time_zone.offset())
        //         .unwrap()
        //         .format(DATE_TIME_FORMAT)
        //         .to_string();
        // let max_timestamp =
        //     NaiveDateTime::parse_from_str(&self.source.meta[MAX_TIMESTAMP], DATE_TIME_FORMAT)
        //         .unwrap()
        //         .and_local_timezone(self.settings.time_zone.offset())
        //         .unwrap()
        //         .format(DATE_TIME_FORMAT)
        //         .to_string();
        let min_timestamp = &self.frame.meta[MIN_TIMESTAMP];
        let max_timestamp = &self.frame.meta[MAX_TIMESTAMP];
        format!("{min_timestamp} {MINUS} {max_timestamp}")
    }

    pub(crate) fn name(&self) -> &str {
        &self.frame.meta[NAME]
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
    pub(super) fn ui(
        &mut self,
        ui: &mut Ui,
        behavior: &mut Behavior,
        tile_id: TileId,
    ) -> UiResponse {
        let id = *self.id.get_or_insert_with(|| ui.next_auto_id());
        let mut state = State::load(ui.ctx(), id);
        let response = TopBottomPanel::top(ui.auto_id_with("Pane"))
            .show_inside(ui, |ui| {
                MenuBar::new()
                    .ui(ui, |ui| {
                        ScrollArea::horizontal()
                            .show(ui, |ui| {
                                ui.set_height(
                                    ui.text_style_height(&TextStyle::Heading) + 4.0 * MARGIN.y,
                                );
                                ui.visuals_mut().button_frame = false;
                                if ui.button(RichText::new(X).heading()).clicked() {
                                    behavior.close = Some(tile_id);
                                }
                                ui.separator();
                                self.top(ui, &mut state)
                            })
                            .inner
                    })
                    .inner
            })
            .inner;
        CentralPanel::default()
            .frame(Frame::central_panel(ui.style()))
            .show_inside(ui, |ui| {
                _ = self.central(ui, &mut state);
                self.windows(ui, &mut state);
            });
        if behavior.close == Some(tile_id) {
            state.remove(ui.ctx(), id);
        } else {
            state.store(ui.ctx(), id);
        }
        if response.dragged() {
            UiResponse::DragStarted
        } else {
            UiResponse::None
        }
    }

    pub(crate) fn header(&mut self, ui: &mut Ui) -> Response {
        let mut response = ui.heading(self.icon()).on_hover_localized(self.name());
        response |= ui.heading(self.title());
        response = response
            .on_hover_text(format!("{:x}", self.frame.hash))
            .on_hover_cursor(CursorIcon::Grab);
        ui.separator();
        // View
        ui.add(ViewWidget::new(&mut self.view));
        ui.separator();

        // Reset
        if ui
            .button(RichText::new(ARROWS_CLOCKWISE).heading())
            .on_hover_localized("reset_table")
            .clicked()
        {
            self.state.reset_table_state = true;
        }
        // Resize
        ui.toggle_value(
            &mut self.settings.table.resizable,
            RichText::new(ARROWS_HORIZONTAL).heading(),
        )
        .on_hover_localized("resize_table");
        ui.separator();
        // Settings
        ui.toggle_value(
            &mut self.state.open_settings_window,
            RichText::new(GEAR).heading(),
        )
        .on_hover_localized("settings");
        ui.separator();
        // Export
        ui.menu_button(RichText::new(FLOPPY_DISK).heading(), |ui| {
            if ui.button("XLSX").clicked() {
                let data_frame = ui.memory_mut(|memory| {
                    let key = TableKey {
                        frame: &self.frame.data,
                        settings: &self.settings,
                    };
                    Hashed {
                        value: memory.caches.cache::<TableComputed>().get(key),
                        hash: hash(key),
                    }
                });
                xlsx::save(&data_frame, "data_frame.xlsx").ok();
                ui.close_menu();
            }
        })
        .response
        .on_hover_localized("save");
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
                PlotView::new(&self.frame.data, &mut self.settings).show(ui);
            }
            View::Table => {
                let data_frame = ui.memory_mut(|memory| {
                    memory.caches.cache::<TableComputed>().get(TableKey {
                        frame: &self.frame.data,
                        settings: &self.settings,
                    })
                });
                TableView::new(&data_frame, &self.settings, &mut self.state).show(ui);
            }
        }
    }

    fn windows(&mut self, ui: &mut Ui) {
        // Settings
        let mut open_settings_window = self.state.open_settings_window;
        let title = match self.view {
            View::Plot => format!("{GEAR} Plot settings"),
            View::Table => format!("{GEAR} Table settings"),
        };
        Window::new(title)
            .id(ui.auto_id_with(ID_SOURCE))
            .default_pos(ui.next_widget_position())
            .open(&mut open_settings_window)
            .show(ui.ctx(), |ui| {
                self.settings.show(ui);
                ui.separator();
                match self.view {
                    View::Plot => self.settings.plot.show(ui),
                    View::Table => self.settings.table.show(ui, &self.frame.data.clone()),
                }
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
pub(crate) mod table;
pub(crate) mod view;
