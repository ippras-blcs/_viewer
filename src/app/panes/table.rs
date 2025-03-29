use super::settings::Settings;
use egui::{Direction, Layout, Response, Sense, Ui, Widget};
use egui_extras::{Column, TableBuilder};
use polars::prelude::*;

/// Table pane
#[derive(Debug, PartialEq)]
pub(crate) struct Pane<'a> {
    pub(crate) data_frame: &'a DataFrame,
    pub(crate) settings: &'a mut Settings,
}

impl Widget for Pane<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        let width = ui.spacing().interact_size.x;
        let height = ui.spacing().interact_size.y;
        let data_frame = self.data_frame;
        TableBuilder::new(ui)
            .cell_layout(Layout::centered_and_justified(Direction::LeftToRight))
            .column(Column::auto_with_initial_suggestion(width))
            .columns(Column::auto(), 2)
            .auto_shrink(false)
            .striped(true)
            .header(height, |mut row| {
                row.col(|ui| {
                    ui.heading("Index");
                });
                row.col(|ui| {
                    ui.heading("Time");
                });
                row.col(|ui| {
                    let text = data_frame.get_column_names()[1].as_str();
                    ui.heading(text);
                });
            })
            .body(|body| {
                let time = data_frame.time();
                let value = data_frame.value();
                let total_rows = data_frame.height();
                body.rows(height, total_rows, |mut row| {
                    let row_index = row.index();
                    // Index
                    row.col(|ui| {
                        ui.label(row_index.to_string());
                    });
                    // Time
                    row.col(|ui| {
                        let text = time.get(row_index).unwrap();
                        ui.label(text);
                    });
                    // Value
                    row.col(|ui| {
                        let text = value.get(row_index).unwrap().to_string();
                        ui.label(text);
                    });
                });
            });
        ui.allocate_response(Default::default(), Sense::hover())
    }
}

impl Pane<'_> {
    pub(crate) fn settings(self, ui: &mut Ui) {
        self.settings.ui(ui)
    }
}

/// Extension methods for [`DataFrame`]
trait DataFrameExt {
    fn time(&self) -> ChunkedArray<StringType>;

    fn try_time(&self) -> PolarsResult<ChunkedArray<StringType>>;

    fn try_value(&self) -> PolarsResult<&ChunkedArray<Float64Type>>;

    fn value(&self) -> &ChunkedArray<Float64Type>;
}

impl DataFrameExt for DataFrame {
    fn time(&self) -> ChunkedArray<StringType> {
        self.try_time().unwrap()
    }

    fn try_time(&self) -> PolarsResult<ChunkedArray<StringType>> {
        // self["Time"].datetime()?.to_string("%Y-%m-%d")
        self["Time"].datetime()?.to_string("%Y-%m-%d %H:%M:%S")
    }

    fn try_value(&self) -> PolarsResult<&ChunkedArray<Float64Type>> {
        self[1].f64()
    }

    fn value(&self) -> &ChunkedArray<Float64Type> {
        self.try_value().unwrap()
    }
}
