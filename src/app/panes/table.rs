use super::{ID_SOURCE, settings::Settings, state::State};
use crate::app::{NAME_TEMPERATURE, NAME_TURBIDITY, YMDHMS};
use egui::{Context, Frame, Id, Margin, RichText, TextStyle, TextWrapMode, Ui, Vec2, vec2};
use egui_l20n::{ResponseExt, UiExt as _};
use egui_phosphor::regular::HASH;
use egui_table::{CellInfo, Column, HeaderCellInfo, HeaderRow, Table, TableDelegate, TableState};
use polars::prelude::*;
use tracing::{error, instrument};

const MARGIN: Vec2 = vec2(4.0, 2.0);

const INDEX: usize = 0;
const IDENTIFIER: usize = 1;
const TIMESTAMP: usize = 2;
const VALUE: usize = 3;
const LEN: usize = 4;

/// Table view
#[derive(Debug)]
pub(crate) struct View<'a> {
    data_frame: &'a DataFrame,
    settings: &'a Settings,
    state: &'a mut State,
}

impl<'a> View<'a> {
    pub(crate) fn new(
        data_frame: &'a DataFrame,
        settings: &'a Settings,
        state: &'a mut State,
    ) -> Self {
        Self {
            data_frame,
            settings,
            state,
        }
    }
}

impl View<'_> {
    pub(super) fn show(&mut self, ui: &mut Ui) {
        let id_salt = Id::new(ID_SOURCE).with("Table");
        if self.state.reset_table_state {
            let id = TableState::id(ui, Id::new(id_salt));
            TableState::reset(ui.ctx(), id);
            self.state.reset_table_state = false;
        }
        let height = ui.text_style_height(&TextStyle::Heading) + 2.0 * MARGIN.y;
        let num_rows = self.data_frame.height() as u64;
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

    fn header_cell_content_ui(&mut self, ui: &mut Ui, row: usize, column: usize) {
        if self.settings.table.truncate {
            ui.style_mut().wrap_mode = Some(TextWrapMode::Truncate);
        }
        match (row, column) {
            (0, INDEX) => {
                ui.heading(HASH).on_hover_localized("index.hover");
            }
            (0, IDENTIFIER) => {
                ui.heading(ui.localize("identifier"))
                    .on_hover_localized("identifier.hover");
            }
            (0, TIMESTAMP) => {
                ui.heading(ui.localize("timestamp"))
                    .on_hover_localized("timestamp.hover");
            }
            (0, VALUE) => {
                ui.heading(ui.localize("value"))
                    .on_hover_localized("value.hover");
            }
            _ => {}
        };
    }

    #[instrument(skip(ui), err)]
    fn cell_content_ui(&mut self, ui: &mut Ui, row: usize, column: usize) -> PolarsResult<()> {
        match (row, column) {
            (row, INDEX) => {
                ui.label(row.to_string());
            }
            (row, IDENTIFIER) => {
                let identifier = self.data_frame["Identifier"].u64()?;
                if let Some(identifier) = identifier.get(row) {
                    ui.label(RichText::new(format!("{identifier:x}")).monospace());
                }
            }
            (row, TIMESTAMP) => {
                let timestamp = self.data_frame["Timestamp"].datetime()?;
                if let Some(timestamp) = timestamp.get(row) {
                    ui.label(self.settings.time_zone.format_time(timestamp, YMDHMS));
                }
            }
            (row, VALUE) => {
                let last = self.data_frame.width() - 1;
                match &*self.data_frame[last].name().to_lowercase() {
                    NAME_TEMPERATURE => {
                        let temperature = self.data_frame[last].f32()?;
                        if let Some(temperature) = temperature.get(row) {
                            ui.label(temperature.to_string());
                        }
                    }
                    NAME_TURBIDITY => {
                        let turbidity = self.data_frame[last].u16()?;
                        if let Some(turbidity) = turbidity.get(row) {
                            ui.label(turbidity.to_string());
                        }
                    }
                    name => {
                        error!("Unsupported name: {name}");
                    }
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
                self.header_cell_content_ui(ui, cell.row_nr, cell.col_range.start)
            });
    }

    fn cell_ui(&mut self, ui: &mut Ui, cell: &CellInfo) {
        if cell.row_nr % 2 == 0 {
            ui.painter()
                .rect_filled(ui.max_rect(), 0.0, ui.visuals().faint_bg_color);
        }
        Frame::new()
            .inner_margin(Margin::from(MARGIN))
            .show(ui, |ui| {
                self.cell_content_ui(ui, cell.row_nr as _, cell.col_nr).ok();
            });
    }

    fn row_top_offset(&self, ctx: &Context, _table_id: Id, row_nr: u64) -> f32 {
        row_nr as f32 * (ctx.style().spacing.interact_size.y + 2.0 * MARGIN.y)
    }
}
