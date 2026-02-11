use self::table::View as TableView;
use super::{Behavior, MARGIN};
use crate::{
    app::{
        computers::turbidity::table::{Computed as TableComputed, Key as TableKey},
        states::turbidity::State,
        widgets::buttons::{MetadataButton, ResetButton, ResizeButton, SettingsButton},
    },
    export,
    utils::{
        hashed::{HashedDataFrame, HashedMetaDataFrame},
        metadata::join,
        polars::format_list_truncated,
    },
};
use anyhow::Result;
use egui::{
    CentralPanel, CursorIcon, Frame, Id, MenuBar, Response, RichText, ScrollArea, TextStyle,
    TextWrapMode, TopBottomPanel, Ui, Widget as _, Window, util::hash,
};
use egui_l20n::prelude::*;
use egui_phosphor::regular::{
    CALCULATOR, ERASER, FLOPPY_DISK, LIST, NOTE_PENCIL, SLIDERS_HORIZONTAL, TAG, TEXT_AA, TRASH, X,
};
use egui_tiles::{TileId, UiResponse};
use metadata::{egui::MetadataWidget, polars::MetaDataFrame};
use polars::prelude::*;
use serde::{Deserialize, Serialize};
use std::{
    fmt::{Debug, Display, from_fn},
    sync::LazyLock,
};
use tracing::instrument;

const ID_SOURCE: &str = "Turbidity";

// pub(crate) static SCHEMA: LazyLock<Schema> = LazyLock::new(|| {
//     Schema::from_iter([
//         Field::new("Label".into(), DataType::String),
//         field!(FATTY_ACID),
//         Field::new(STEREOSPECIFIC_NUMBERS123.into(), DataType::Float64),
//         Field::new(STEREOSPECIFIC_NUMBERS2.into(), DataType::Float64),
//     ])
// });

/// Turbidity pane
#[derive(Default, Deserialize, Serialize)]
pub(crate) struct Pane {
    id: Option<Id>,
    frames: Vec<HashedMetaDataFrame>,
}

impl Pane {
    pub(crate) fn new(frames: Vec<HashedMetaDataFrame>) -> Self {
        Self { id: None, frames }
    }

    pub(crate) fn title(&self) -> String {
        self.title_with_separator(" ")
    }

    fn title_with_separator(&self, separator: &str) -> String {
        format_list_truncated(&self.frames, separator)
    }

    fn id(&self) -> impl Display {
        from_fn(|f| {
            if let Some(id) = self.id {
                write!(f, "{id:?}-")?;
            }
            write!(f, "{}", hash(&self.frames))
        })
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
                self.central(ui, &mut state);
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

    fn top(&mut self, ui: &mut Ui, state: &mut State) -> Response {
        let mut response = ui.heading(NOTE_PENCIL).on_hover_localized("Turbidity");
        response |= ui.heading(self.title());
        response = response
            .on_hover_text(self.id().to_string())
            .on_hover_cursor(CursorIcon::Grab);
        ui.separator();
        ResetButton::new(&mut state.settings.table.reset).ui(ui);
        ResizeButton::new(&mut state.settings.table.resizable).ui(ui);
        ui.separator();
        SettingsButton::new(&mut state.windows.open_settings).ui(ui);
        ui.separator();
        MetadataButton::new(&mut state.windows.open_metadata).ui(ui);
        ui.separator();
        self.save_button(ui, state);
        ui.separator();
        response
    }

    // Save button
    fn save_button(&self, ui: &mut Ui, state: &State) {
        ui.menu_button(RichText::new(FLOPPY_DISK).heading(), |ui| {
            // let name = self.frames[state.settings.index].meta.format(".");
            // if ui
            //     .button((FLOPPY_DISK, "RON"))
            //     .on_hover_localized("Save")
            //     .on_hover_ui(|ui| {
            //         ui.label(format!("{name}.fa.utca.ron"));
            //     })
            //     .clicked()
            // {
            //     _ = self.save_ron(&name, state);
            // }
        });
    }

    #[instrument(skip(self, state), err)]
    fn save_ron(&self, name: impl Debug + Display, state: &State) -> Result<()> {
        // export::ron::save(
        //     &self.frames[state.settings.index],
        //     &format!("{name}.blcs.ron"),
        // )
        Ok(())
    }

    fn central(&mut self, ui: &mut Ui, state: &mut State) {
        let frame = ui.memory_mut(|memory| {
            memory
                .caches
                .cache::<TableComputed>()
                .get(TableKey::new(&self.frames, &state.settings))
        });
        TableView::new(&frame, &mut state.settings).show(ui);
    }
}

impl Pane {
    fn windows(&mut self, ui: &mut Ui, state: &mut State) {
        self.metadata_window(ui, state);
        self.settings_window(ui, state);
    }

    fn metadata_window(&mut self, ui: &mut Ui, state: &mut State) {
        Window::new(format!("{TAG} Turbidity metadata"))
            .id(ui.auto_id_with(ID_SOURCE).with("Metadata"))
            .default_pos(ui.next_widget_position())
            .open(&mut state.windows.open_metadata)
            .show(ui.ctx(), |ui| {
                MetadataWidget::new(&join(&self.frames)).show(ui);
            });
    }

    fn settings_window(&mut self, ui: &mut Ui, state: &mut State) {
        if let Some(inner_response) =
            Window::new(format!("{SLIDERS_HORIZONTAL} Turbidity settings"))
                .id(ui.auto_id_with(ID_SOURCE).with("Settings"))
                .default_pos(ui.next_widget_position())
                .open(&mut state.windows.open_settings)
                .show(ui.ctx(), |ui| {
                    state.settings.show(ui);
                })
        {
            inner_response
                .response
                .on_hover_text(self.title())
                .on_hover_text(self.id().to_string());
        }
    }
}

mod table;
