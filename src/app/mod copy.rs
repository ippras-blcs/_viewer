use self::panes::{
    central::{Behavior, Pane, TreeExt},
    left::SettingsPane,
};
use anyhow::Result;
use blc::Timed;
use eframe::{get_value, set_value, CreationContext, Storage, APP_KEY};
use egui::{
    global_dark_light_mode_switch, menu::bar, warn_if_debug_build, Align, Align2, CentralPanel,
    Color32, DroppedFile, Id, LayerId, Layout, Order, RichText, SidePanel, TextStyle,
    TopBottomPanel, Ui,
};
use egui_ext::{DroppedFileExt, HoveredFileExt};
use egui_tiles::Tree;
use panes::central::Ddoc;
use polars::prelude::*;
use ron::de;
use serde::{Deserialize, Serialize};
use std::{fmt::Write, future::Future, str, time::Duration};
use tracing::{error, info, trace};

const MQTT_ID: &str = "ippras.ru/blc/viewer";
const MQTT_HOST: &str = "broker.emqx.io";
const MQTT_PORT: u16 = 1883;
const MQTT_TOPIC: &str = "ippras.ru/blc/#";
const MQTT_TOPIC_TEMPERATURE: &str = "ippras.ru/blc/temperature";
const MQTT_TOPIC_TURBIDITY: &str = "ippras.ru/blc/turbidity";
const MQTT_TOPIC_DDOC_C1: &str = "ippras.ru/blc/ddoc/c1"; // mA
const MQTT_TOPIC_DDOC_C2: &str = "ippras.ru/blc/ddoc/c2"; // mA
const MQTT_TOPIC_DDOC_T1: &str = "ippras.ru/blc/ddoc/t1"; // °C
const MQTT_TOPIC_DDOC_T2: &str = "ippras.ru/blc/ddoc/t2"; // °C
const MQTT_TOPIC_DDOC_V1: &str = "ippras.ru/blc/ddoc/v1"; // mg/L
const MQTT_TOPIC_DDOC_V2: &str = "ippras.ru/blc/ddoc/v2"; // %
const NAME_TEMPERATURE: &str = "Temperature";
const NAME_TURBIDITY: &str = "Turbidity";
const NAME_DDOC_C1: &str = "DDOC.C1";
const NAME_DDOC_C2: &str = "DDOC.C2";
const NAME_DDOC_T1: &str = "DDOC.T1";
const NAME_DDOC_T2: &str = "DDOC.T2";
const NAME_DDOC_V1: &str = "DDOC.V1";
const NAME_DDOC_V2: &str = "DDOC.V2";

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(Deserialize, Serialize)]
#[serde(default)]
pub struct App {
    left_panel: bool,
    tree: Tree<Pane>,
    #[serde(skip)]
    behavior: Behavior,
}

impl Default for App {
    fn default() -> Self {
        Self {
            left_panel: true,
            tree: Tree::default(),
            behavior: Default::default(),
        }
    }
}

impl App {
    /// Called once before the first frame.
    pub fn new(cc: &CreationContext) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        spawn_mqtt(&cc.egui_ctx);

        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        if let Some(storage) = cc.storage {
            if let Some(app) = get_value(storage, APP_KEY) {
                return app;
            }
        }
        Default::default()
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
                match deserialize(&dropped_file) {
                    Ok(data_frame) if data_frame.width() > 1 => {
                        trace!(?data_frame);
                        match data_frame[1].name() {
                            NAME_TEMPERATURE => {
                                self.tree.insert_pane(Pane::Temperature(Some(data_frame)))
                            }
                            NAME_TURBIDITY => {
                                self.tree.insert_pane(Pane::Turbidity(Some(data_frame)))
                            }
                            NAME_DDOC_C1 => self
                                .tree
                                .insert_pane(Pane::Ddoc(Ddoc::C1(Some(data_frame)))),
                            NAME_DDOC_C2 => self
                                .tree
                                .insert_pane(Pane::Ddoc(Ddoc::C2(Some(data_frame)))),
                            NAME_DDOC_T1 => self
                                .tree
                                .insert_pane(Pane::Ddoc(Ddoc::T1(Some(data_frame)))),
                            NAME_DDOC_T2 => self
                                .tree
                                .insert_pane(Pane::Ddoc(Ddoc::T2(Some(data_frame)))),
                            NAME_DDOC_V1 => self
                                .tree
                                .insert_pane(Pane::Ddoc(Ddoc::V1(Some(data_frame)))),
                            NAME_DDOC_V2 => self
                                .tree
                                .insert_pane(Pane::Ddoc(Ddoc::V2(Some(data_frame)))),
                            _ => {
                                error!("Unsupported format");
                                continue;
                            }
                        };
                    }
                    error => {
                        if let Err(error) = error {
                            error!(%error);
                        }
                        continue;
                    }
                };
            }
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
            self.tree.ui(&mut self.behavior, ui);
            if let Some(id) = self.behavior.close {
                self.tree.tiles.remove(id);
            }
        });
    }

    // Left panel
    fn left_panel(&mut self, ctx: &egui::Context) {
        SidePanel::left("left_panel")
            .frame(egui::Frame::side_top_panel(&ctx.style()))
            .resizable(true)
            .show_animated(ctx, self.left_panel, |ui| {
                SettingsPane::view(ui);
            });
    }

    // Top panel
    fn top_panel(&mut self, ctx: &egui::Context) {
        TopBottomPanel::top("top_panel").show(ctx, |ui| {
            bar(ui, |ui| {
                // Left panel
                let text = if self.left_panel { "⮪" } else { "⮩" };
                ui.toggle_value(&mut self.left_panel, text)
                    .on_hover_text("Left panel");
                ui.separator();
                ui.visuals_mut().button_frame = false;
                global_dark_light_mode_switch(ui);
                ui.separator();
                if ui.button("🔃").on_hover_text("Reset").clicked() {
                    *self = Default::default();
                    ui.memory_mut(|memory| *memory = Default::default());
                }
                ui.separator();
                // // Temperature
                // let temperature = self.tree.tiles.find_pane(&Pane::Temperature(None));
                // if ui
                //     .selectable_label(temperature.is_some(), "Te")
                //     .on_hover_text("Temperature")
                //     .clicked()
                // {
                //     if let Some(id) = temperature {
                //         self.tree.tiles.remove(id);
                //     } else {
                //         self.tree.insert_pane(Pane::Temperature(None));
                //     }
                // }
                // // Turbidity
                // let turbidity = self.tree.tiles.find_pane(&Pane::Turbidity(None));
                // if ui
                //     .selectable_label(turbidity.is_some(), "Tu")
                //     .on_hover_text("Turbidity")
                //     .clicked()
                // {
                //     if let Some(id) = turbidity {
                //         self.tree.tiles.remove(id);
                //     } else {
                //         self.tree.insert_pane(Pane::Turbidity(None));
                //     }
                // }
                // In real time
                let mut toggle = |ui: &mut Ui, pane| {
                    let tile_id = self.tree.tiles.find_pane(&pane);
                    if ui
                        .selectable_label(tile_id.is_some(), pane.name())
                        .on_hover_text(pane.name())
                        .clicked()
                    {
                        if let Some(id) = tile_id {
                            self.tree.tiles.remove(id);
                        } else {
                            self.tree.insert_pane(pane);
                        }
                    }
                };
                ui.menu_button("🕐", |ui| {
                    // Temperature
                    toggle(ui, Pane::Temperature(None));
                    toggle(ui, Pane::Turbidity(None));
                    // DDOC
                    ui.menu_button("DDOC", |ui| {
                        toggle(ui, Pane::Ddoc(Ddoc::V1(None)));
                        toggle(ui, Pane::Ddoc(Ddoc::T1(None)));
                        toggle(ui, Pane::Ddoc(Ddoc::V2(None)));
                        toggle(ui, Pane::Ddoc(Ddoc::C1(None)));
                        toggle(ui, Pane::Ddoc(Ddoc::C2(None)));
                        toggle(ui, Pane::Ddoc(Ddoc::T2(None)));
                        // ui.close_menu();
                    });

                    // let temperature = self.tree.tiles.find_pane(&Pane::Temperature(None));
                    // if ui
                    //     .selectable_label(temperature.is_some(), NAME_TEMPERATURE)
                    //     .on_hover_text("Temperature")
                    //     .clicked()
                    // {
                    //     if let Some(id) = temperature {
                    //         self.tree.tiles.remove(id);
                    //     } else {
                    //         self.tree.insert_pane(Pane::Temperature(None));
                    //     }
                    // }
                    // // Turbidity
                    // let turbidity = self.tree.tiles.find_pane(&Pane::Turbidity(None));
                    // if ui
                    //     .selectable_label(turbidity.is_some(), NAME_TURBIDITY)
                    //     .on_hover_text("Turbidity")
                    //     .clicked()
                    // {
                    //     if let Some(id) = turbidity {
                    //         self.tree.tiles.remove(id);
                    //     } else {
                    //         self.tree.insert_pane(Pane::Turbidity(None));
                    //     }
                    // }
                    // // DDOC
                    // ui.menu_button("DDOC", |ui| {
                    //     ui.checkbox(&mut true, NAME_DDOC_C1);
                    //     ui.checkbox(&mut true, NAME_DDOC_C2);
                    //     ui.checkbox(&mut true, NAME_DDOC_T1);
                    //     ui.checkbox(&mut true, NAME_DDOC_T2);
                    //     ui.checkbox(&mut true, NAME_DDOC_V1);
                    //     ui.checkbox(&mut true, NAME_DDOC_V2);
                    //     // ui.close_menu();
                    // });
                })
                .response
                .on_hover_text("In real time");
                // Add
                ui.menu_button("➕", |ui| {
                    // ui.close_menu();
                });
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
        ctx.request_repaint();
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn spawn<F: Future<Output = ()> + Send + 'static>(f: F) {
    std::thread::spawn(move || futures::executor::block_on(f));
}

#[cfg(target_arch = "wasm32")]
fn spawn<F: Future<Output = ()> + 'static>(f: F) {
    wasm_bindgen_futures::spawn_local(f);
}

#[cfg(target_arch = "wasm32")]
fn spawn_mqtt(ctx: &egui::Context) {
    let (mut sender, receiver) = loop {
        // broker.emqx.io:8084
        // match ewebsock::connect("wss://broker.emqx.io:8084/mqtt", Default::default()) {
        match ewebsock::connect("wss://echo.websocket.org", Default::default()) {
            Ok((sender, receiver)) => break (sender, receiver),
            Err(error) => error!(%error),
        }
    };
    spawn(async move {
        // sender.send(ewebsock::WsMessage::Text("Hello!".into()));
        loop {
            sender.send(ewebsock::WsMessage::Text("Hello!".into()));
        }
    });
    spawn(async move {
        // sender.send(ewebsock::WsMessage::Text("Hello!".into()));
        while let Some(event) = receiver.try_recv() {
            println!("Received {:?}", event);
        }
    });
}

#[cfg(not(target_arch = "wasm32"))]
fn spawn_mqtt(ctx: &egui::Context) {
    use blc::MICROSECONDS;
    use polars::datatypes::TimeUnit;
    use ron::de;
    use rumqttc::{Client, Event, Incoming, MqttOptions, QoS};

    let mut options = MqttOptions::new(MQTT_ID, MQTT_HOST, MQTT_PORT);
    options.set_keep_alive(Duration::from_secs(9));
    let (client, mut connection) = Client::new(options, 9);
    let context = ctx.clone();
    spawn(async move {
        if let Err(error) = (|| -> Result<()> {
            client.subscribe(MQTT_TOPIC, QoS::ExactlyOnce)?;
            for event in connection.iter() {
                if let Event::Incoming(Incoming::Publish(publish)) = event? {
                    let Timed { time, value }: Timed<f64> = de::from_bytes(&publish.payload)?;
                    trace!(?time);
                    let timestamp = (time.unix_timestamp_nanos() / MICROSECONDS) as i64;
                    let time = AnyValue::Datetime(timestamp, TimeUnit::Milliseconds, &None);
                    let name = match &*publish.topic {
                        MQTT_TOPIC_TEMPERATURE => NAME_TEMPERATURE,
                        MQTT_TOPIC_TURBIDITY => NAME_TURBIDITY,
                        MQTT_TOPIC_DDOC_C1 => NAME_DDOC_C1,
                        MQTT_TOPIC_DDOC_C2 => NAME_DDOC_C2,
                        MQTT_TOPIC_DDOC_T1 => NAME_DDOC_T1,
                        MQTT_TOPIC_DDOC_T2 => NAME_DDOC_T2,
                        MQTT_TOPIC_DDOC_V1 => NAME_DDOC_V1,
                        MQTT_TOPIC_DDOC_V2 => NAME_DDOC_V2,
                        topic => {
                            error!("Unexpected MQTT topic {topic}");
                            continue;
                        }
                    };
                    let row = &df! {
                        "Time" => vec![time],
                        name => vec![value],
                    }?;
                    let id = Id::new(&*publish.topic);
                    let mut data_frame = context
                        .memory(|memory| memory.data.get_temp::<DataFrame>(id))
                        .unwrap_or_default();
                    data_frame = data_frame.vstack(&row)?;
                    context.memory_mut(|memory| {
                        memory.data.insert_temp(id, data_frame);
                    });
                }
            }
            Ok(())
        })() {
            error!(%error);
        }
    });
}

fn deserialize(dropped_file: &DroppedFile) -> Result<DataFrame> {
    Ok(de::from_bytes(&dropped_file.bytes()?)?)
}

// mod computers;
mod context;
mod panes;
