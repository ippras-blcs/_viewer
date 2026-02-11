use super::ID_SOURCE;
use crate::{
    app::{
        NAME_TEMPERATURE, YMDHMS,
        states::turbidity::{State, settings::Settings},
    },
    r#const::{EM_DASH, IDENTIFIER, TIMESTAMP, TURBIDITY},
    utils::hashed::HashedDataFrame,
};
use egui::{Context, Frame, Id, Margin, RichText, TextStyle, TextWrapMode, Ui, Vec2, vec2};
use egui_l20n::{ResponseExt, UiExt as _};
use egui_phosphor::regular::HASH;
use egui_table::{CellInfo, Column, HeaderCellInfo, HeaderRow, Table, TableDelegate, TableState};
use polars::prelude::*;
use std::ops::Range;
use tracing::{error, instrument};

const MARGIN: Vec2 = vec2(4.0, 2.0);

// const INDEX: usize = 0;
// const IDENTIFIER: usize = 1;
// const TIMESTAMP: usize = 2;
// const VALUE: usize = 3;
const LEN: usize = 4;

/// Turbidity table view
#[derive(Debug)]
pub(crate) struct View<'a> {
    frame: &'a HashedDataFrame,
    settings: &'a mut Settings,
}

impl<'a> View<'a> {
    pub(crate) fn new(frame: &'a HashedDataFrame, settings: &'a mut Settings) -> Self {
        Self { frame, settings }
    }
}

impl View<'_> {
    pub(super) fn show(&mut self, ui: &mut Ui) {
        let id_salt = Id::new(ID_SOURCE).with("Table");
        if self.settings.table.reset {
            let id = TableState::id(ui, Id::new(id_salt));
            TableState::reset(ui.ctx(), id);
            self.settings.table.reset = false;
        }
        let height = ui.text_style_height(&TextStyle::Heading) + 2.0 * MARGIN.y;
        let num_rows = self.frame.height() as u64;
        let num_columns = LEN;
        Table::new()
            .id_salt(id_salt)
            .num_rows(num_rows)
            .columns(vec![
                Column::default()
                    .resizable(self.settings.table.resizable);
                num_columns
            ])
            .num_sticky_cols(self.settings.table.sticky_columns)
            .headers([HeaderRow::new(height)])
            // .auto_size_mode(AutoSizeMode::OnParentResize)
            .show(ui, self);
    }

    fn header_cell_content_ui(&mut self, ui: &mut Ui, row: usize, column: Range<usize>) {
        if self.settings.table.truncate {
            ui.style_mut().wrap_mode = Some(TextWrapMode::Truncate);
        }
        match (row, column) {
            (0, top::INDEX) => {
                ui.heading(HASH).on_hover_localized("index.hover");
            }
            (0, top::IDENTIFIER) => {
                ui.heading(ui.localize("identifier"))
                    .on_hover_localized("identifier.hover");
            }
            (0, top::TIMESTAMP) => {
                ui.heading(ui.localize("timestamp"))
                    .on_hover_localized("timestamp.hover");
            }
            (0, top::TURBIDITY) => {
                ui.heading(ui.localize("value"))
                    .on_hover_localized("value.hover");
            }
            _ => {}
        };
    }

    #[instrument(skip(ui), err)]
    fn cell_content_ui(
        &mut self,
        ui: &mut Ui,
        row: usize,
        column: Range<usize>,
    ) -> PolarsResult<()> {
        match (row, column) {
            (row, top::INDEX) => {
                ui.label(row.to_string());
            }
            (row, top::TIMESTAMP) => {
                let timestamp = self.frame[TIMESTAMP].datetime()?;
                if let Some(timestamp) = timestamp.phys.get(row) {
                    ui.label(self.settings.time_zone.format_time(timestamp, YMDHMS));
                } else {
                    ui.label(EM_DASH);
                }
            }
            (row, top::IDENTIFIER) => {
                let identifier = self.frame[IDENTIFIER].u64()?;
                if let Some(identifier) = identifier.get(row) {
                    ui.label(RichText::new(format!("{identifier:x}")).monospace());
                } else {
                    ui.label(EM_DASH);
                }
            }
            (row, top::TURBIDITY) => {
                let turbidity = self.frame[TURBIDITY].u16()?;
                if let Some(turbidity) = turbidity.get(row) {
                    ui.label(turbidity.to_string());
                } else {
                    ui.label(EM_DASH);
                }
            }
            _ => {}
        }
        Ok(())
    }
}

impl TableDelegate for View<'_> {
    fn header_cell_ui(&mut self, ui: &mut Ui, cell: &HeaderCellInfo) {
        Frame::new()
            .inner_margin(Margin::from(MARGIN))
            .show(ui, |ui| {
                self.header_cell_content_ui(ui, cell.row_nr, cell.col_range.clone())
            });
    }

    fn cell_ui(&mut self, ui: &mut Ui, cell: &CellInfo) {
        if cell.row_nr.is_multiple_of(2) {
            ui.painter()
                .rect_filled(ui.max_rect(), 0.0, ui.visuals().faint_bg_color);
        }
        Frame::new()
            .inner_margin(Margin::from(MARGIN))
            .show(ui, |ui| {
                _ = self.cell_content_ui(ui, cell.row_nr as _, cell.col_nr..cell.col_nr + 1);
            });
    }

    fn row_top_offset(&self, ctx: &Context, _table_id: Id, row_nr: u64) -> f32 {
        row_nr as f32 * (ctx.style().spacing.interact_size.y + 2.0 * MARGIN.y)
    }
}

mod top {
    use super::*;

    pub(super) const INDEX: Range<usize> = 0..1;
    pub(super) const TIMESTAMP: Range<usize> = INDEX.end..INDEX.end + 1;
    pub(super) const IDENTIFIER: Range<usize> = TIMESTAMP.end..TIMESTAMP.end + 1;
    pub(super) const TURBIDITY: Range<usize> = IDENTIFIER.end..IDENTIFIER.end + 1;
}
