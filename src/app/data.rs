use super::{
    metadata::{FILE, ICON, MAX_TIMESTAMP, MIN_TIMESTAMP, NAME},
    panes::Kind,
};
use crate::{
    app::{metadata::MetaDataFrame, panes::Pane},
    utils::hashed::Hashed,
};
use egui::{CentralPanel, Grid, Label, RichText, ScrollArea, Sense, TopBottomPanel, Ui, menu::bar};
use egui_extras::{Column, TableBuilder};
use egui_l20n::{ResponseExt, UiExt as _};
use egui_phosphor::regular::{BROWSERS, CHECK, MINUS, TRASH};
use egui_tiles::Tree;
use egui_tiles_ext::{TreeExt, VERTICAL};
use indexmap::IndexSet;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// Data
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub(crate) struct Data {
    pub(crate) frames: IndexSet<MetaDataFrame>,
    pub(crate) selected: HashSet<MetaDataFrame>,
}

impl Data {
    pub(crate) fn selected(&self) -> Vec<MetaDataFrame> {
        self.frames
            .iter()
            .filter_map(|frame| self.selected.contains(frame).then_some(frame.clone()))
            .collect()
    }

    pub(crate) fn add(&mut self, mut frame: MetaDataFrame) {
        frame.data.rechunk_mut();
        self.frames.insert(frame);
        self.frames.sort_by(|left, right| {
            left.meta[NAME]
                .cmp(&right.meta[NAME])
                .then(left.meta[MIN_TIMESTAMP].cmp(&right.meta[MIN_TIMESTAMP]))
                .then(left.meta[MAX_TIMESTAMP].cmp(&right.meta[MAX_TIMESTAMP]))
        });
    }
}

impl Data {
    pub(crate) fn show(&mut self, ui: &mut Ui, tree: &mut Tree<Pane>) {
        // Header
        TopBottomPanel::top(ui.auto_id_with("TopPanel")).show_inside(ui, |ui| {
            bar(ui, |ui| {
                ScrollArea::horizontal().show(ui, |ui| {
                    self.header(ui, tree);
                })
            })
        });
        // Body
        CentralPanel::default().show_inside(ui, |ui| {
            self.body(ui);
        });
    }

    fn header(&mut self, ui: &mut Ui, tree: &mut Tree<Pane>) {
        ui.heading(ui.localize("loaded_files"))
            .on_hover_localized("loaded_files.hover");
        ui.separator();
        // Toggle all
        if ui
            .button(RichText::new(CHECK).heading())
            .on_hover_localized("toggle_all")
            .on_hover_localized("toggle_all.hover")
            .clicked()
        {
            if self.selected.is_empty() {
                self.selected = self.frames.iter().cloned().collect();
            } else {
                self.selected.clear();
            }
        }
        ui.separator();
        // Delete all
        if ui
            .button(RichText::new(TRASH).heading())
            .on_hover_localized("delete_all")
            .clicked()
        {
            *self = Default::default();
        }
        ui.separator();
        if ui
            .button(RichText::new(BROWSERS).heading())
            .on_hover_localized("browse")
            .clicked()
        {
            let frames = self.selected();
            for frame in frames {
                let pane = Pane {
                    kind: Kind::Dtec,
                    source: Hashed::new(frame),
                    target: Default::default(),
                    table_settings: Default::default(),
                    plot_settings: Default::default(),
                    state: Default::default(),
                    view: Default::default(),
                };
                tree.insert_pane::<VERTICAL>(pane);
            }
        }
        ui.separator();
    }

    fn body(&mut self, ui: &mut Ui) {
        // ui.visuals_mut().widgets.inactive.bg_fill = Color32::TRANSPARENT;
        // ui.style_mut().wrap_mode = Some(TextWrapMode::Truncate);
        let height = ui.spacing().interact_size.y;
        let mut delete = None;
        TableBuilder::new(ui)
            .auto_shrink(false)
            .column(Column::auto().resizable(false))
            .column(Column::exact(height))
            .column(Column::auto().resizable(true))
            .column(Column::exact(height))
            .body(|mut body| {
                for (index, frame) in self.frames.iter().enumerate() {
                    let mut changed = false;
                    body.row(height, |mut row| {
                        // Index
                        row.col(|ui| {
                            ui.label(index.to_string());
                        });
                        // Checkbox
                        row.col(|ui| {
                            let mut checked = self.selected.contains(frame);
                            let response = ui.checkbox(&mut checked, "");
                            changed |= response.changed();
                        });
                        // Label
                        row.col(|ui| {
                            let text = format!("{} {}", frame.meta[ICON], frame.meta[FILE],);
                            let response = ui
                                .add(Label::new(text).sense(Sense::click()).truncate())
                                .on_hover_ui(|ui| {
                                    ui.label(format!(
                                        "{} {MINUS} {}",
                                        frame.meta[MIN_TIMESTAMP], frame.meta[MAX_TIMESTAMP],
                                    ));
                                })
                                // .on_hover_ui(|ui| {
                                //     MetadataWidget::new(&frame.meta).show(ui);
                                // })
                                .on_hover_ui(|ui| {
                                    Grid::new(ui.next_auto_id()).show(ui, |ui| {
                                        ui.label("Rows");
                                        ui.label(frame.data.height().to_string());
                                        ui.end_row();
                                        ui.label("Columns");
                                        ui.label(frame.data.width().to_string());
                                        ui.end_row();
                                    });
                                });
                            changed |= response.clicked();
                        });
                        // Delete
                        row.col(|ui| {
                            if ui.button(TRASH).clicked() {
                                delete = Some(frame.clone());
                            }
                        });
                    });
                    if changed {
                        if body.ui_mut().input(|input| input.modifiers.command) {
                            if self.selected.contains(frame) {
                                self.selected.remove(frame);
                            } else {
                                self.selected.insert(frame.clone());
                            }
                        } else {
                            if self.selected.contains(frame) {
                                self.selected.remove(&frame);
                            } else {
                                self.selected.insert(frame.clone());
                            }
                        }
                    }
                }
            });
        if let Some(frame) = &delete {
            self.frames.remove(frame);
            self.selected.remove(frame);
        }
    }
}

// impl Data {
//     pub(crate) fn show(&mut self, ui: &mut Ui, tree: &mut Tree<Pane>) {
//         // Header
//         bar(ui, |ui| {
//             ui.heading(ui.localize("loaded_files"))
//                 .on_hover_localized("loaded_files.hover");
//             ui.separator();
//             // Toggle all
//             if ui
//                 .button(RichText::new(CHECK).heading())
//                 .on_hover_localized("toggle_all")
//                 .on_hover_localized("toggle_all.hover")
//                 .clicked()
//             {
//                 if self.selected.is_empty() {
//                     self.selected = self.frames.iter().cloned().collect();
//                 } else {
//                     self.selected.clear();
//                 }
//             }
//             ui.separator();
//             // Delete all
//             if ui
//                 .button(RichText::new(TRASH).heading())
//                 .on_hover_localized("delete_all")
//                 .clicked()
//             {
//                 *self = Default::default();
//             }
//             ui.separator();
//             if ui
//                 .button(RichText::new("Pane::icon").heading())
//                 .on_hover_localized("show")
//                 .clicked()
//             {
//                 let frames = self.selected();
//                 for frame in frames {
//                     let pane = Pane {
//                         kind: Kind::Dtec,
//                         source: Some(frame.data),
//                         target: Default::default(),
//                         settings: Default::default(),
//                         state: Default::default(),
//                         view: Default::default(),
//                     };
//                     tree.insert_pane::<VERTICAL>(pane);
//                     println!("self.tree: {:?}", tree);
//                 }
//             }
//             ui.separator();
//         });
//         // Body
//         ui.separator();
//         ui.visuals_mut().widgets.inactive.bg_fill = Color32::TRANSPARENT;
//         let mut swap = None;
//         let mut delete = None;
//         let height = ui.spacing().interact_size.y;
//         ui.dnd_drop_zone::<usize, ()>(Frame::new(), |ui| {
//             // ui.style_mut().wrap_mode = Some(TextWrapMode::Truncate);
//             TableBuilder::new(ui)
//                 .auto_shrink(false)
//                 .column(Column::auto().resizable(false))
//                 .column(Column::exact(height))
//                 .column(Column::auto().resizable(true))
//                 .column(Column::exact(height))
//                 .body(|mut body| {
//                     for (index, frame) in self.frames.iter().enumerate() {
//                         let mut changed = false;
//                         body.row(height, |mut row| {
//                             row.col(|ui| {
//                                 let response = ui
//                                     .dnd_drag_source(ui.auto_id_with(index), index, |ui| {
//                                         ui.label(index.to_string())
//                                     })
//                                     .response;
//                                 // Detect drops onto this item
//                                 if let (Some(pointer), Some(hovered_payload)) = (
//                                     ui.input(|input| input.pointer.interact_pos()),
//                                     response.dnd_hover_payload::<usize>(),
//                                 ) {
//                                     let rect = response.rect;
//                                     // Preview insertion:
//                                     let stroke = Stroke::new(1.0, Color32::WHITE);
//                                     let to = if *hovered_payload == index {
//                                         // We are dragged onto ourselves
//                                         ui.painter().hline(rect.x_range(), rect.center().y, stroke);
//                                         index
//                                     } else if pointer.y < rect.center().y {
//                                         // Above us
//                                         ui.painter().hline(rect.x_range(), rect.top(), stroke);
//                                         index
//                                     } else {
//                                         // Below us
//                                         ui.painter().hline(rect.x_range(), rect.bottom(), stroke);
//                                         index + 1
//                                     };
//                                     if let Some(from) = response.dnd_release_payload() {
//                                         // The user dropped onto this item.
//                                         swap = Some((*from, to));
//                                     }
//                                 }
//                             });
//                             // Checkbox
//                             row.col(|ui| {
//                                 let mut checked = self.selected.contains(frame);
//                                 let response = ui.checkbox(&mut checked, "");
//                                 changed |= response.changed();
//                             });
//                             // Label
//                             row.col(|ui| {
//                                 let text = if let Some(version) = &frame.meta.version {
//                                     &format!("{} {version}", frame.meta.name)
//                                 } else {
//                                     &frame.meta.name
//                                 };
//                                 let response = ui
//                                     .add(Label::new(text).sense(Sense::click()).truncate())
//                                     .on_hover_ui(|ui| {
//                                         MetadataWidget::new(&frame.meta).show(ui);
//                                     })
//                                     .on_hover_ui(|ui| {
//                                         Grid::new(ui.next_auto_id()).show(ui, |ui| {
//                                             ui.label("Rows");
//                                             ui.label(frame.data.height().to_string());
//                                             ui.end_row();
//                                             ui.label("Columns");
//                                             ui.label(frame.data.width().to_string());
//                                             ui.end_row();
//                                         });
//                                     });
//                                 changed |= response.clicked();
//                             });
//                             // Delete
//                             row.col(|ui| {
//                                 if ui.button(TRASH).clicked() {
//                                     delete = Some(index);
//                                 }
//                             });
//                         });
//                         if changed {
//                             if body.ui_mut().input(|input| input.modifiers.command) {
//                                 if self.selected.contains(frame) {
//                                     self.selected.remove(frame);
//                                 } else {
//                                     self.selected.insert(frame.clone());
//                                 }
//                             } else {
//                                 if self.selected.contains(frame) {
//                                     self.selected.remove(&frame);
//                                 } else {
//                                     self.selected.insert(frame.clone());
//                                 }
//                             }
//                         }
//                     }
//                 });
//         });
//         if let Some((from, to)) = swap {
//             if from != to {
//                 let frame = self.frames.remove(from);
//                 if from < to {
//                     self.frames.insert(to - 1, frame);
//                 } else {
//                     self.frames.insert(to, frame);
//                 }
//             }
//         }
//         if let Some(index) = delete {
//             self.selected.remove(&self.frames.remove(index));
//         }
//     }
// }
