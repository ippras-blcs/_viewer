use self::{
    cloud::GoogleDrive,
    data::Data,
    panes::{Ddoc, Pane, behavior::Behavior},
};
use crate::{app::metadata::MetaDataFrame, localization::ContextExt as _};
use anyhow::{Error, Result};
use arrow::temporal_conversions::timestamp_ms_to_datetime;
use eframe::{APP_KEY, CreationContext, Storage, get_value, set_value};
use egui::{
    Align, Align2, CentralPanel, Color32, ComboBox, DroppedFile, FontDefinitions, Id, LayerId,
    Layout, Order, RichText, ScrollArea, SidePanel, Spinner, TextStyle, TextWrapMode,
    TopBottomPanel, Ui, menu::bar, warn_if_debug_build,
};
use egui_ext::{DroppedFileExt, HoveredFileExt, LightDarkButton};
use egui_l20n::{ResponseExt as _, UiExt};
use egui_phosphor::{
    Variant, add_to_fonts,
    regular::{
        ARROW_FAT_LEFT, ARROW_FAT_RIGHT, ARROWS_CLOCKWISE, CLOCK, CLOUD_ARROW_DOWN, DROP_HALF,
        GRID_FOUR, QUESTION, ROCKET, SIDEBAR, SIDEBAR_SIMPLE, SQUARE_SPLIT_HORIZONTAL,
        SQUARE_SPLIT_VERTICAL, TABS, THERMOMETER, TRANSLATE, TRASH,
    },
};
use egui_tiles::{ContainerKind, Tile, Tree};
use egui_tiles_ext::{TilesExt as _, TreeExt as _, VERTICAL};
use metadata::{FILE, ICON, MAX_TIMESTAMP, MIN_TIMESTAMP, NAME};
use panes::Kind;
use polars::prelude::*;
use rumqttc::tokio_rustls::rustls::crypto::hmac::Key;
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    fmt::Write,
    future::Future,
    io::Cursor,
    str,
    sync::mpsc::{Receiver, Sender, channel},
};
use tracing::{error, info, instrument, trace};

const NAME_DDOC_C1: &str = "DDOC.C1";
const NAME_DDOC_C2: &str = "DDOC.C2";
const NAME_DDOC_T1: &str = "DDOC.T1";
const NAME_DDOC_T2: &str = "DDOC.T2";
const NAME_DDOC_V1: &str = "DDOC.V1";
const NAME_DDOC_V2: &str = "DDOC.V2";
const NAME_TEMPERATURE: &str = "temperature";
const NAME_TURBIDITY: &str = "turbidity";

const DATE_TIME_FORMAT: &str = "%Y-%m-%d %H:%M:%S";
const MAX_PRECISION: usize = 16;
const ICON_SIZE: f32 = 32.0;

macro icon($icon:expr) {
    RichText::new($icon).size(ICON_SIZE)
}

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(Deserialize, Serialize)]
#[serde(default)]
pub struct App {
    left_panel: bool,
    reactive: bool,

    tree: Tree<Pane>,
    data: Data,

    #[serde(skip)]
    google_drive: GoogleDrive,
    #[serde(skip)]
    data_receiver: Receiver<DataFrame>,
    #[serde(skip)]
    error_sender: Sender<Error>,
    #[serde(skip)]
    error_receiver: Receiver<Error>,
}

impl Default for App {
    fn default() -> Self {
        // let (data_sender, data_receiver) = channel(9);
        // let (error_sender, error_receiver) = channel(9);
        let (data_sender, data_receiver) = channel();
        let (error_sender, error_receiver) = channel();
        Self {
            reactive: true,
            left_panel: true,
            tree: Tree::empty("tree"),
            data: Default::default(),
            google_drive: GoogleDrive::new(data_sender, error_sender.clone()),
            data_receiver,
            error_sender,
            error_receiver,
        }
    }
}

impl App {
    /// Called once before the first frame.
    pub fn new(cc: &CreationContext) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.
        let mut fonts = FontDefinitions::default();
        add_to_fonts(&mut fonts, Variant::Regular);
        cc.egui_ctx.set_fonts(fonts);
        cc.egui_ctx.set_localizations();
        mqtt::spawn(&cc.egui_ctx);

        // return Default::default();
        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        cc.storage
            .and_then(|storage| get_value(storage, APP_KEY))
            .unwrap_or_default()
    }

    fn drag_and_drop(&mut self, ctx: &egui::Context) {
        // Preview hovering files
        if let Some(text) = ctx.input(|input| {
            (!input.raw.hovered_files.is_empty()).then(|| {
                let mut text = "Dropping files:".to_owned();
                for file in &input.raw.hovered_files {
                    write!(text, "\n{}", file.display()).ok();
                }
                text
            })
        }) {
            let painter =
                ctx.layer_painter(LayerId::new(Order::Foreground, Id::new("file_drop_target")));
            let screen_rect = ctx.screen_rect();
            painter.rect_filled(screen_rect, 0.0, Color32::from_black_alpha(192));
            painter.text(
                screen_rect.center(),
                Align2::CENTER_CENTER,
                text,
                TextStyle::Heading.resolve(&ctx.style()),
                Color32::WHITE,
            );
        }
        // Parse dropped files
        if let Some(dropped_files) = ctx.input(|input| {
            (!input.raw.dropped_files.is_empty()).then_some(input.raw.dropped_files.clone())
        }) {
            info!(?dropped_files);
            for dropped_file in dropped_files {
                if let Ok(frame) = deserialize(&dropped_file) {
                    trace!(?frame);
                    self.data.add(frame);
                }
            }
        }
    }

    fn data(&mut self) {
        // while let Ok(data_frame) = self.data_receiver.try_recv() {
        //     let kind = match data_frame[1].name().as_str() {
        //         NAME_TEMPERATURE => Kind::Dtec,
        //         NAME_TURBIDITY => Kind::Atuc,
        //         NAME_DDOC_C1 => Kind::Ddoc(Ddoc::C1),
        //         NAME_DDOC_C2 => Kind::Ddoc(Ddoc::C2),
        //         NAME_DDOC_T1 => Kind::Ddoc(Ddoc::T1),
        //         NAME_DDOC_T2 => Kind::Ddoc(Ddoc::T2),
        //         NAME_DDOC_V1 => Kind::Ddoc(Ddoc::V1),
        //         NAME_DDOC_V2 => Kind::Ddoc(Ddoc::V2),
        //         _ => {
        //             error!("Unsupported format");
        //             continue;
        //         }
        //     };
        //     self.tree.insert_pane::<VERTICAL>(Pane {
        //         kind,
        //         source: Some(data_frame),
        //         target: Default::default(),
        //         settings: Default::default(),
        //         state: Default::default(),
        //         view: Default::default(),
        //     });
        // }
    }

    fn error(&mut self) {
        // while let Some(error) = self.error_receiver.recv().await {
        //     error!(%error);
        // }
        while let Ok(error) = self.error_receiver.try_recv() {
            error!(%error);
        }
    }
}

impl App {
    fn panels(&mut self, ctx: &egui::Context) {
        self.top_panel(ctx);
        self.bottom_panel(ctx);
        self.left_panel(ctx);
        self.central_panel(ctx);
    }

    // Bottom panel
    fn bottom_panel(&mut self, ctx: &egui::Context) {
        TopBottomPanel::bottom("bottom_panel").show(ctx, |ui| {
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                warn_if_debug_build(ui);
                ui.label(RichText::new(env!("CARGO_PKG_VERSION")).small());
                ui.separator();
            });
        });
    }

    // Central panel
    fn central_panel(&mut self, ctx: &egui::Context) {
        CentralPanel::default().show(ctx, |ui| {
            let mut behavior = Behavior::new();
            self.tree.ui(&mut behavior, ui);
            if let Some(id) = behavior.close.take() {
                self.tree.tiles.remove(id);
            }
        });
    }

    // Left panel
    fn left_panel(&mut self, ctx: &egui::Context) {
        SidePanel::left("LeftPanel")
            .resizable(true)
            .show_animated(ctx, self.left_panel, |ui| {
                ScrollArea::vertical().show(ui, |ui| {
                    self.data.show(ui, &mut self.tree);
                });
            });
    }

    // Top panel
    fn top_panel(&mut self, ctx: &egui::Context) {
        TopBottomPanel::top("top_panel").show(ctx, |ui| {
            bar(ui, |ui| {
                // Left panel
                ui.toggle_value(&mut self.left_panel, icon!(SIDEBAR_SIMPLE))
                    .on_hover_text(ui.localize("left_panel"));
                ui.separator();
                ui.light_dark_button(ICON_SIZE);
                ui.separator();
                ui.toggle_value(&mut self.reactive, icon!(ROCKET))
                    .on_hover_text("reactive")
                    .on_hover_text(ui.localize("reactive_description_enabled"))
                    .on_disabled_hover_text(ui.localize("reactive_description_disabled"));
                ui.separator();
                if ui
                    .button(icon!(TRASH))
                    .on_hover_text(ui.localize("reset_application"))
                    .clicked()
                {
                    *self = Default::default();
                }
                ui.separator();
                if ui
                    .button(icon!(ARROWS_CLOCKWISE))
                    .on_hover_text(ui.localize("reset_gui"))
                    .clicked()
                {
                    ui.memory_mut(|memory| *memory = Default::default());
                    ui.ctx().set_localizations();
                }
                ui.separator();
                if ui
                    .button(icon!(SQUARE_SPLIT_VERTICAL))
                    .on_hover_text(ui.localize("vertical"))
                    .clicked()
                {
                    if let Some(id) = self.tree.root {
                        if let Some(Tile::Container(container)) = self.tree.tiles.get_mut(id) {
                            container.set_kind(ContainerKind::Vertical);
                        }
                    }
                }
                if ui
                    .button(icon!(SQUARE_SPLIT_HORIZONTAL))
                    .on_hover_text(ui.localize("horizontal"))
                    .clicked()
                {
                    if let Some(id) = self.tree.root {
                        if let Some(Tile::Container(container)) = self.tree.tiles.get_mut(id) {
                            container.set_kind(ContainerKind::Horizontal);
                        }
                    }
                }
                if ui
                    .button(icon!(GRID_FOUR))
                    .on_hover_text(ui.localize("grid"))
                    .clicked()
                {
                    if let Some(id) = self.tree.root {
                        if let Some(Tile::Container(container)) = self.tree.tiles.get_mut(id) {
                            container.set_kind(ContainerKind::Grid);
                        }
                    }
                }
                if ui
                    .button(icon!(TABS))
                    .on_hover_text(ui.localize("tabs"))
                    .clicked()
                {
                    if let Some(id) = self.tree.root {
                        if let Some(Tile::Container(container)) = self.tree.tiles.get_mut(id) {
                            container.set_kind(ContainerKind::Tabs);
                        }
                    }
                }
                ui.separator();
                // In real time
                let mut toggle = |ui: &mut Ui, pane: Pane| {
                    let tile_id = self.tree.tiles.find_pane_by(|candidate| {
                        candidate.kind == pane.kind
                            && candidate.is_real_time() == pane.is_real_time()
                    });
                    if ui
                        .selectable_label(tile_id.is_some(), ui.localize(pane.text()))
                        .on_hover_text(ui.localize(pane.hover_text()))
                        .clicked()
                    {
                        if let Some(id) = tile_id {
                            self.tree.tiles.remove(id);
                        } else {
                            self.tree.insert_pane::<VERTICAL>(pane);
                        }
                    }
                };
                ui.menu_button(icon!(CLOCK), |ui| {
                    // Temperature
                    toggle(ui, Pane::DTEC);
                    toggle(ui, Pane::ATUC);
                    // DDOC
                    ui.menu_button(
                        ui.localize("digital_disolved_oxygen_controller.abbreviation"),
                        |ui| {
                            toggle(ui, Pane::DDOC_V1);
                            toggle(ui, Pane::DDOC_V2);
                            toggle(ui, Pane::DDOC_T1);
                            toggle(ui, Pane::DDOC_T2);
                            toggle(ui, Pane::DDOC_C1);
                            toggle(ui, Pane::DDOC_C2);
                        },
                    )
                    .response
                    .on_disabled_hover_localized("digital_disolved_oxygen_controller.hover");
                })
                .response
                .on_hover_text(ui.localize("in_real_time"));
                // // Open cloud saved
                // ui.menu_button(icon!(CLOUD_ARROW_DOWN), |ui| {
                //     self.google_drive.ui(ui);
                // })
                // .response
                // .on_hover_text(ui.localize("cloud_saved"));

                ui.separator();
                // Locale
                ui.locale_button();
            });
        });
    }
}

impl eframe::App for App {
    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn Storage) {
        set_value(storage, APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.panels(ctx);
        self.drag_and_drop(ctx);
        self.data();
        self.error();
        // localization::update(ctx);
        if self.reactive {
            ctx.request_repaint();
        }
    }
}

#[instrument(err)]
fn deserialize(dropped_file: &DroppedFile) -> Result<MetaDataFrame> {
    let bytes = dropped_file.bytes()?;
    let mut reader = ParquetReader::new(Cursor::new(bytes));
    let meta = reader.get_metadata()?;
    // if let Some(meta) = &meta.key_value_metadata {
    //     for key_value in meta {
    //         println!("name: {} {:?}", key_value.key, key_value.value);
    //     }
    // }
    // let mut meta = Metadata::default();
    let mut meta = BTreeMap::new();
    meta.insert(FILE.to_owned(), dropped_file.name().to_owned());
    let data = reader.finish()?;
    let last = data.width() - 1;
    let name = data[last].name().to_lowercase();
    // Icon
    match &*name {
        NAME_TEMPERATURE => meta.insert(ICON.to_owned(), THERMOMETER.to_owned()),
        NAME_TURBIDITY => meta.insert(ICON.to_owned(), DROP_HALF.to_owned()),
        _ => meta.insert(ICON.to_owned(), QUESTION.to_owned()),
    };
    // Timestamp
    if let Some((min, max)) = data["Timestamp"].datetime()?.min_max() {
        if let Some(min) = timestamp_ms_to_datetime(min) {
            meta.insert(
                MIN_TIMESTAMP.to_owned(),
                min.format(DATE_TIME_FORMAT).to_string(),
            );
        }
        if let Some(max) = timestamp_ms_to_datetime(max) {
            meta.insert(
                MAX_TIMESTAMP.to_owned(),
                max.format(DATE_TIME_FORMAT).to_string(),
            );
        }
    }
    // Name
    meta.insert(NAME.to_owned(), name);
    Ok(MetaDataFrame::new(meta, data))
}

#[cfg(not(target_arch = "wasm32"))]
fn spawn<F: Future<Output = ()> + Send + 'static>(f: F) {
    std::thread::spawn(move || futures::executor::block_on(f));
}

#[cfg(target_arch = "wasm32")]
fn spawn<F: Future<Output = ()> + 'static>(f: F) {
    wasm_bindgen_futures::spawn_local(f);
}

mod cloud;
mod computers;
mod data;
mod metadata;
mod mqtt;
mod panes;
