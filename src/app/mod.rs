use self::{data::Data, panes::Pane};
use crate::{
    app::{
        panes::Behavior,
        states::State,
        widgets::{
            about::About,
            buttons::{
                AboutButton, GridButton, HorizontalButton, LeftPanelButton, ReactiveButton,
                ResetButton, SettingsButton, TabsButton, VerticalButton,
            },
        },
    },
    r#const::{IDENTIFIER, KIND, TIMESTAMP, TURBIDITY, TYPE},
    localization::ContextExt as _,
    utils::{
        hashed::{HashedDataFrame, HashedMetaDataFrame},
        metadata::{Parsed, parse},
    },
};
use anyhow::{Error, Result, anyhow};
use arrow::temporal_conversions::timestamp_ms_to_datetime;
use eframe::{APP_KEY, CreationContext, Storage, get_value, set_value};
use egui::{
    Align, Align2, CentralPanel, Color32, ComboBox, Context, DroppedFile, FontDefinitions, Id,
    LayerId, Layout, MenuBar, Order, RichText, ScrollArea, SidePanel, Spinner, TextStyle,
    TextWrapMode, TopBottomPanel, Ui, Widget, Window, menu::bar, warn_if_debug_build,
};
use egui_ext::{DroppedFileExt, HoveredFileExt, LightDarkButton};
use egui_l20n::{ResponseExt as _, UiExt};
use egui_phosphor::{
    Variant, add_to_fonts,
    regular::{
        ARROW_FAT_LEFT, ARROW_FAT_RIGHT, ARROWS_CLOCKWISE, CLOCK, CLOUD_ARROW_DOWN, DROP_HALF,
        GRID_FOUR, INFO, QUESTION, ROCKET, SIDEBAR, SIDEBAR_SIMPLE, SLIDERS_HORIZONTAL,
        SQUARE_SPLIT_HORIZONTAL, SQUARE_SPLIT_VERTICAL, TABS, THERMOMETER, TRANSLATE, TRASH,
    },
};
use egui_tiles::{ContainerKind, Tile, Tree};
use egui_tiles_ext::{TilesExt as _, TreeExt as _, VERTICAL};
use metadata::{AUTHORS, DATE, Metadata, NAME, PARAMETERS, VERSION, polars::MetaDataFrame};
use polars::prelude::*;
use protocol::meta::Metadata as ProtocolMetadata;
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    fmt::Write,
    future::Future,
    io::{BufRead, Cursor, Seek, SeekFrom},
    str,
    sync::{
        LazyLock,
        mpsc::{Receiver, Sender, channel},
    },
};
use tracing::{error, info, instrument, trace};
use urlencoding::encode;

const ID_SOURCE: &str = "BLCS";

const NAME_DDOC_C1: &str = "DDOC.C1";
const NAME_DDOC_C2: &str = "DDOC.C2";
const NAME_DDOC_T1: &str = "DDOC.T1";
const NAME_DDOC_T2: &str = "DDOC.T2";
const NAME_DDOC_V1: &str = "DDOC.V1";
const NAME_DDOC_V2: &str = "DDOC.V2";
const NAME_TEMPERATURE: &str = "temperature";
// const NAME_TURBIDITY: &str = "turbidity";

const YMDHMSZ: &str = "%Y-%m-%d %H:%M:%S %Z";
const YMDHMS: &str = "%Y-%m-%d %H:%M:%S";
const MAX_PRECISION: usize = 16;
const ICON_SIZE: f32 = 32.0;

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(Deserialize, Serialize)]
#[serde(default)]
pub struct App {
    data: Data,
    tree: Tree<Pane>,
}

impl Default for App {
    fn default() -> Self {
        Self {
            data: Default::default(),
            tree: Tree::empty("Tree"),
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
        // mqtt::spawn(&cc.egui_ctx);

        // return Default::default();
        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        cc.storage
            .and_then(|storage| get_value(storage, APP_KEY))
            .unwrap_or_default()
    }
}

impl App {
    fn panels(&mut self, ctx: &Context, state: &mut State) {
        self.top_panel(ctx, state);
        self.bottom_panel(ctx);
        self.left_panel(ctx, state);
        self.central_panel(ctx);
    }

    // Bottom panel
    fn bottom_panel(&mut self, ctx: &Context) {
        TopBottomPanel::bottom("BottomPanel").show(ctx, |ui| {
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                warn_if_debug_build(ui);
                ui.label(RichText::new(env!("CARGO_PKG_VERSION")).small());
                ui.separator();
            });
        });
    }

    // Central panel
    fn central_panel(&mut self, ctx: &Context) {
        CentralPanel::default().show(ctx, |ui| {
            let mut behavior = Behavior { close: None };
            self.tree.ui(&mut behavior, ui);
            if let Some(id) = behavior.close {
                self.tree.tiles.remove(id);
            }
        });
    }

    // Left panel
    fn left_panel(&mut self, ctx: &Context, state: &mut State) {
        SidePanel::left("LeftPanel").resizable(true).show_animated(
            ctx,
            state.settings.left_panel,
            |ui| {
                ScrollArea::vertical().show(ui, |ui| {
                    self.data.show(ui);
                });
            },
        );
    }

    // Top panel
    fn top_panel(&mut self, ctx: &Context, state: &mut State) {
        TopBottomPanel::top("TopPanel").show(ctx, |ui| {
            MenuBar::new().ui(ui, |ui| {
                ScrollArea::horizontal().show(ui, |ui| {
                    LeftPanelButton::new(&mut state.settings.left_panel)
                        .size(ICON_SIZE)
                        .ui(ui);
                    ui.separator();
                    ReactiveButton::new(&mut state.settings.reactive)
                        .size(ICON_SIZE)
                        .ui(ui);
                    ui.separator();
                    // Light/Dark
                    ui.light_dark_button(ICON_SIZE);
                    ui.separator();
                    ResetButton::new(&mut state.settings.reset)
                        .size(ICON_SIZE)
                        .ui(ui);
                    ui.separator();
                    self.layouts(ui, state);
                    ui.separator();
                    SettingsButton::new(&mut state.windows.open_settings)
                        .size(ICON_SIZE)
                        .ui(ui);
                    ui.separator();
                    // Github.ui(ui);
                    ui.separator();
                    AboutButton::new(&mut state.windows.open_about)
                        .size(ICON_SIZE)
                        .ui(ui);
                    ui.separator();
                });
            });
        });
    }

    fn layouts(&mut self, ui: &mut Ui, state: &mut State) {
        VerticalButton::new(&mut state.settings.layout.container_kind)
            .size(ICON_SIZE)
            .ui(ui);
        HorizontalButton::new(&mut state.settings.layout.container_kind)
            .size(ICON_SIZE)
            .ui(ui);
        GridButton::new(&mut state.settings.layout.container_kind)
            .size(ICON_SIZE)
            .ui(ui);
        TabsButton::new(&mut state.settings.layout.container_kind)
            .size(ICON_SIZE)
            .ui(ui);
    }

    // // Top panel
    // fn top_panel(&mut self, ctx: &Context) {
    //     TopBottomPanel::top("top_panel").show(ctx, |ui| {
    //         bar(ui, |ui| {
    //             // Left panel
    //             ui.toggle_value(
    //                 &mut self.left_panel,
    //                 RichText::new(SIDEBAR_SIMPLE).size(ICON_SIZE),
    //             )
    //             .on_hover_text(ui.localize("left_panel"));
    //             ui.separator();
    //             ui.light_dark_button(ICON_SIZE);
    //             ui.separator();
    //             ui.toggle_value(&mut self.reactive, RichText::new(ROCKET).size(ICON_SIZE))
    //                 .on_hover_text("reactive")
    //                 .on_hover_text(ui.localize("reactive_description_enabled"))
    //                 .on_disabled_hover_text(ui.localize("reactive_description_disabled"));
    //             ui.separator();
    //             if ui
    //                 .button(RichText::new(TRASH).size(ICON_SIZE))
    //                 .on_hover_text(ui.localize("reset_application"))
    //                 .clicked()
    //             {
    //                 *self = Default::default();
    //             }
    //             ui.separator();
    //             if ui
    //                 .button(RichText::new(ARROWS_CLOCKWISE).size(ICON_SIZE))
    //                 .on_hover_text(ui.localize("reset_gui"))
    //                 .clicked()
    //             {
    //                 ui.memory_mut(|memory| *memory = Default::default());
    //                 ui.ctx().set_localizations();
    //             }
    //             ui.separator();
    //             if ui
    //                 .button(RichText::new(SQUARE_SPLIT_VERTICAL).size(ICON_SIZE))
    //                 .on_hover_text(ui.localize("vertical"))
    //                 .clicked()
    //             {
    //                 if let Some(id) = self.tree.root {
    //                     if let Some(Tile::Container(container)) = self.tree.tiles.get_mut(id) {
    //                         container.set_kind(ContainerKind::Vertical);
    //                     }
    //                 }
    //             }
    //             if ui
    //                 .button(RichText::new(SQUARE_SPLIT_HORIZONTAL).size(ICON_SIZE))
    //                 .on_hover_text(ui.localize("horizontal"))
    //                 .clicked()
    //             {
    //                 if let Some(id) = self.tree.root {
    //                     if let Some(Tile::Container(container)) = self.tree.tiles.get_mut(id) {
    //                         container.set_kind(ContainerKind::Horizontal);
    //                     }
    //                 }
    //             }
    //             if ui
    //                 .button(RichText::new(GRID_FOUR).size(ICON_SIZE))
    //                 .on_hover_text(ui.localize("grid"))
    //                 .clicked()
    //             {
    //                 if let Some(id) = self.tree.root {
    //                     if let Some(Tile::Container(container)) = self.tree.tiles.get_mut(id) {
    //                         container.set_kind(ContainerKind::Grid);
    //                     }
    //                 }
    //             }
    //             if ui
    //                 .button(RichText::new(TABS).size(ICON_SIZE))
    //                 .on_hover_text(ui.localize("tabs"))
    //                 .clicked()
    //             {
    //                 if let Some(id) = self.tree.root {
    //                     if let Some(Tile::Container(container)) = self.tree.tiles.get_mut(id) {
    //                         container.set_kind(ContainerKind::Tabs);
    //                     }
    //                 }
    //             }
    //             ui.separator();
    //             // In real time
    //             // let mut toggle = |ui: &mut Ui, pane: Pane| {
    //             //     let tile_id = self.tree.tiles.find_pane_by(|candidate| {
    //             //         candidate.kind == pane.kind
    //             //             && candidate.is_real_time() == pane.is_real_time()
    //             //     });
    //             //     if ui
    //             //         .selectable_label(tile_id.is_some(), ui.localize(pane.text()))
    //             //         .on_hover_text(ui.localize(pane.hover_text()))
    //             //         .clicked()
    //             //     {
    //             //         if let Some(id) = tile_id {
    //             //             self.tree.tiles.remove(id);
    //             //         } else {
    //             //             self.tree.insert_pane::<VERTICAL>(pane);
    //             //         }
    //             //     }
    //             // };
    //             // ui.menu_button(RichText::new(CLOCK).size(ICON_SIZE), |ui| {
    //             //     // Temperature
    //             //     toggle(ui, Pane::DTEC);
    //             //     toggle(ui, Pane::ATUC);
    //             //     // DDOC
    //             //     ui.menu_button(
    //             //         ui.localize("digital_disolved_oxygen_controller.abbreviation"),
    //             //         |ui| {
    //             //             toggle(ui, Pane::DDOC_V1);
    //             //             toggle(ui, Pane::DDOC_V2);
    //             //             toggle(ui, Pane::DDOC_T1);
    //             //             toggle(ui, Pane::DDOC_T2);
    //             //             toggle(ui, Pane::DDOC_C1);
    //             //             toggle(ui, Pane::DDOC_C2);
    //             //         },
    //             //     )
    //             //     .response
    //             //     .on_disabled_hover_localized("digital_disolved_oxygen_controller.hover");
    //             // })
    //             // .response
    //             // .on_hover_text(ui.localize("in_real_time"));
    //             // // Open cloud saved
    //             // ui.menu_button(RichText::new(CLOUD_ARROW_DOWN).size(ICON_SIZE), |ui| {
    //             //     self.google_drive.ui(ui);
    //             // })
    //             // .response
    //             // .on_hover_text(ui.localize("cloud_saved"));
    //             ui.separator();
    //             // Locale
    //             ui.locale_button();
    //         });
    //     });
    // }
}

// Windows
impl App {
    fn windows(&mut self, ctx: &Context, state: &mut State) {
        self.about_window(ctx, state);
        self.settings_window(ctx, state);
    }

    fn about_window(&mut self, ctx: &Context, state: &mut State) {
        Window::new(format!("{INFO} About"))
            .open(&mut state.windows.open_about)
            .show(ctx, |ui| About.ui(ui));
    }

    fn settings_window(&mut self, ctx: &Context, state: &mut State) {
        Window::new(format!("{SLIDERS_HORIZONTAL} Settings"))
            .open(&mut state.windows.open_settings)
            .show(ctx, |ui| {
                state.settings.show(ui);
            });
    }
}

// Copy/Paste, Drag&Drop
impl App {
    fn data(&mut self, ctx: &Context, state: &mut State) {
        if let Some(frame) =
            ctx.data_mut(|data| data.remove_temp::<HashedMetaDataFrame>(Id::new("Data")))
        {
            self.data.add(frame);
            state.settings.left_panel = true;
        }
    }

    fn browse(&mut self, ctx: &Context) {
        if let Some(frames) =
            ctx.data_mut(|data| data.remove_temp::<Vec<HashedMetaDataFrame>>(Id::new("Browse")))
        {
            self.tree.insert_pane::<VERTICAL>(Pane::turbidity(frames));
        }
    }

    fn drag_and_drop(&mut self, ctx: &Context) {
        // Preview hovering files
        if let Some(text) = ctx.input(|input| {
            (!input.raw.hovered_files.is_empty()).then(|| {
                let mut text = String::from("Dropping files:");
                for file in &input.raw.hovered_files {
                    write!(text, "\n{}", file.display()).ok();
                }
                text
            })
        }) {
            let painter =
                ctx.layer_painter(LayerId::new(Order::Foreground, Id::new("file_drop_target")));
            let content_rect = ctx.content_rect();
            painter.rect_filled(content_rect, 0.0, Color32::from_black_alpha(192));
            painter.text(
                content_rect.center(),
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
                _ = self.parse(ctx, dropped_file);
            }
        }
    }

    #[instrument(skip_all, err)]
    fn parse(&mut self, ctx: &Context, dropped_file: DroppedFile) -> Result<()> {
        // /// Turbidity schema
        // static TURBIDITY_SCHEMA: LazyLock<SchemaRef> = LazyLock::new(|| {
        //     Arc::new(Schema::from_iter([
        //         Field::new(
        //             PlSmallStr::from_static(TIMESTAMP),
        //             DataType::Datetime(TimeUnit::Milliseconds, None),
        //         ),
        //         Field::new(PlSmallStr::from_static(IDENTIFIER), DataType::UInt64),
        //         Field::new(PlSmallStr::from_static(TURBIDITY), DataType::UInt16),
        //     ]))
        // });
        /// Turbidity schema
        static TURBIDITY_SCHEMA: LazyLock<SchemaRef> = LazyLock::new(|| {
            Arc::new(Schema::from_iter([
                Field::new(
                    PlSmallStr::from_static(TIMESTAMP),
                    DataType::Datetime(TimeUnit::Milliseconds, None),
                ),
                Field::new(PlSmallStr::from_static(TURBIDITY), DataType::UInt16),
            ]))
        });

        // /// Metadata
        // #[derive(Debug, Deserialize, Serialize)]
        // struct MetadataStruct {
        //     authors: Vec<String>,
        //     identifier: u64,
        //     name: String,
        //     date: String,
        //     r#type: String,
        // }

        let bytes = dropped_file.bytes()?;
        trace!(?bytes);
        let mut reader = Cursor::new(&bytes);
        let mut buffer = String::new();
        reader.read_line(&mut buffer)?;
        // (authors:["Kazakov Giorgi Vladimirovich", "Sidorov Roman Alexandrovich"], date_time:"2026-02-16T20:25:17.769191+03:00", identifier:2522015810364357672, kind:Temperature, name:"The Name")
        let deserialized = ron::de::from_bytes::<ProtocolMetadata>(&bytes)?;
        println!("deserialized: {}", ron::to_string(&deserialized)?);
        let mut meta = Metadata::new();
        meta.insert(AUTHORS.to_owned(), deserialized.authors.join(";"));
        meta.insert(IDENTIFIER.to_owned(), deserialized.identifier.to_string());
        meta.insert(NAME.to_owned(), deserialized.name.to_owned());
        meta.insert(DATE.to_owned(), deserialized.date_time.to_string());
        meta.insert(KIND.to_owned(), format!("{:?}", deserialized.kind));
        println!("meta: {meta}");

        // // Turbidity{Authors=KGV;SRA;Identifier=c0a80094;Type=Turbidity}.2026-02-11-20-36-22
        // // (authors0["Kazakov Giorgi Vladimirovich","Sidorov Roman Alexandrovich"],identifier03232235668,name0"TheName",date0"2026-02-11-20-36-22",type0"Turbidity")
        // let t = MyStruct {
        //     authors: vec![
        //         "Kazakov Giorgi Vladimirovich".to_owned(),
        //         "Sidorov Roman Alexandrovich".to_owned(),
        //     ],
        //     identifier: 0xc0a80094,
        //     name: "TheName".to_owned(),
        //     date: "2026-02-11-20-36-22".to_owned(),
        //     r#type: "Turbidity".to_owned(),
        // };
        // let deserialized = ron::from_bytes()?;
        // let encoded = encode(&serialized);

        // Data
        // let data = LazyCsvReader::new_with_sources(ScanSources::Buffers(Arc::new([bytes.into()])))
        //     .with_has_header(true)
        //     .with_schema(Some(TURBIDITY_SCHEMA.clone()))
        //     .finish()?
        //     .with_column(lit(&*meta[IDENTIFIER]).alias(IDENTIFIER))
        //     .collect()?;
        let data = CsvReadOptions::default()
            .with_schema(Some(TURBIDITY_SCHEMA.clone()))
            .with_has_header(false)
            .with_skip_rows(1)
            .into_reader_with_file_handle(reader)
            .finish()?;
        println!("data: {data:?}");
        let frame = MetaDataFrame::new(meta, HashedDataFrame::new(data)?);
        let schema = frame.data.schema();
        if TURBIDITY_SCHEMA.matches_schema(schema).is_ok() {
            // .is_ok_and(|cast| !cast)
            info!("TURBIDITY");
            self.data.add(frame);
        } else {
            return Err(
                polars_err!(SchemaMismatch: r#"Invalid dropped file schema: expected [`TURBIDITY`], got = `{schema:?}`"#),
            )?;
        }

        // let mut reader = ParquetReader::new(Cursor::new(bytes));
        // // let meta = reader.get_metadata()?;
        // let mut meta = Metadata::default();
        // // meta.insert(FILE.to_owned(), dropped_file.name().to_owned());
        // let data = reader.finish()?;
        // let last = data.width() - 1;
        // let name = data[last].name().to_lowercase();
        // // Name
        // meta.insert(NAME.to_owned(), name);

        // let frame = MetaDataFrame::new(meta, HashedDataFrame::new(data)?);
        // let schema = frame.data.schema();
        // if TURBIDITY_SCHEMA
        //     .matches_schema(schema)
        //     .is_ok_and(|cast| !cast)
        // {
        //     info!("TURBIDITY");
        //     self.data.add(frame);
        // } else {
        //     return Err(
        //         polars_err!(SchemaMismatch: r#"Invalid dropped file schema: expected [`TURBIDITY`], got = `{schema:?}`"#),
        //     )?;
        // }
        Ok(())
    }

    // fn data(&mut self) {
    //     // while let Ok(data_frame) = self.data_receiver.try_recv() {
    //     //     let kind = match data_frame[1].name().as_str() {
    //     //         NAME_TEMPERATURE => Kind::Dtec,
    //     //         NAME_TURBIDITY => Kind::Atuc,
    //     //         NAME_DDOC_C1 => Kind::Ddoc(Ddoc::C1),
    //     //         NAME_DDOC_C2 => Kind::Ddoc(Ddoc::C2),
    //     //         NAME_DDOC_T1 => Kind::Ddoc(Ddoc::T1),
    //     //         NAME_DDOC_T2 => Kind::Ddoc(Ddoc::T2),
    //     //         NAME_DDOC_V1 => Kind::Ddoc(Ddoc::V1),
    //     //         NAME_DDOC_V2 => Kind::Ddoc(Ddoc::V2),
    //     //         _ => {
    //     //             error!("Unsupported format");
    //     //             continue;
    //     //         }
    //     //     };
    //     //     self.tree.insert_pane::<VERTICAL>(Pane {
    //     //         kind,
    //     //         source: Some(data_frame),
    //     //         target: Default::default(),
    //     //         settings: Default::default(),
    //     //         state: Default::default(),
    //     //         view: Default::default(),
    //     //     });
    //     // }
    // }

    fn state(&mut self, ctx: &Context, state: &mut State) {
        if state.settings.reset {
            *self = Default::default();
            // Cache
            let caches = ctx.memory_mut(|memory| memory.caches.clone());
            ctx.memory_mut(|memory| {
                memory.caches = caches;
            });
            ctx.set_localizations();
            state.settings.reset = false;
        }
        if let Some(container_kind) = state.settings.layout.container_kind.take()
            && let Some(id) = self.tree.root
            && let Some(Tile::Container(container)) = self.tree.tiles.get_mut(id)
        {
            container.set_kind(container_kind);
        }
        if state.settings.reactive {
            ctx.request_repaint();
        }
    }

    fn error(&mut self) {
        // while let Some(error) = self.error_receiver.recv().await {
        //     error!(%error);
        // }
        // while let Ok(error) = self.error_receiver.try_recv() {
        //     error!(%error);
        // }
    }
}

impl eframe::App for App {
    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn Storage) {
        set_value(storage, APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        let mut state = State::load(ctx, Id::new(ID_SOURCE));
        self.data(ctx, &mut state);
        self.browse(ctx);
        // Pre update
        self.panels(ctx, &mut state);
        self.windows(ctx, &mut state);
        // Post update
        self.drag_and_drop(ctx);
        self.state(ctx, &mut state);
        state.store(ctx, Id::new(ID_SOURCE));
    }
}

// #[instrument(err)]
// fn deserialize(dropped_file: &DroppedFile) -> Result<HashedMetaDataFrame> {
//     let bytes = dropped_file.bytes()?;
//     let mut reader = ParquetReader::new(Cursor::new(bytes));
//     let meta = reader.get_metadata()?;
//     // if let Some(meta) = &meta.key_value_metadata {
//     //     for key_value in meta {
//     //         println!("name: {} {:?}", key_value.key, key_value.value);
//     //     }
//     // }
//     let mut meta = Metadata::default();
//     // meta.insert(FILE.to_owned(), dropped_file.name().to_owned());
//     let data = reader.finish()?;
//     let last = data.width() - 1;
//     let name = data[last].name().to_lowercase();
//     // // Icon
//     // match &*name {
//     //     NAME_TEMPERATURE => meta.insert(ICON.to_owned(), THERMOMETER.to_owned()),
//     //     NAME_TURBIDITY => meta.insert(ICON.to_owned(), DROP_HALF.to_owned()),
//     //     _ => meta.insert(ICON.to_owned(), QUESTION.to_owned()),
//     // };
//     // // Timestamp
//     // if let Some((min, max)) = data[TIMESTAMP].datetime()?.phys.min_max() {
//     //     if let Some(min) = timestamp_ms_to_datetime(min) {
//     //         meta.insert(MIN_TIMESTAMP.to_owned(), min.format(YMDHMS).to_string());
//     //     }
//     //     if let Some(max) = timestamp_ms_to_datetime(max) {
//     //         meta.insert(MAX_TIMESTAMP.to_owned(), max.format(YMDHMS).to_string());
//     //     }
//     // }
//     // Name
//     meta.insert(NAME.to_owned(), name);

//     Ok(MetaDataFrame::new(meta, HashedDataFrame::new(data)?))
// }

mod cloud;
mod computers;
mod data;
mod mqtt;

pub mod panes;
pub mod states;
pub mod widgets;
