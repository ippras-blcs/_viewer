use anyhow::Result;
use arrow::temporal_conversions::*;
use polars::prelude::*;
use rust_xlsxwriter::{Format, Table, TableColumn, Workbook, worksheet::Worksheet};
use tracing::instrument;

#[cfg(not(target_arch = "wasm32"))]
pub use self::native::save;
#[cfg(target_arch = "wasm32")]
pub use self::web::save;

#[cfg(not(target_arch = "wasm32"))]
mod native {
    use super::*;

    #[cfg(not(target_arch = "wasm32"))]
    #[instrument(err)]
    pub fn save(data_frame: &DataFrame, name: &str) -> Result<()> {
        let mut workbook = Workbook::new();
        write(data_frame, workbook.add_worksheet())?;
        workbook.save(name)?;
        Ok(())
    }
}

#[cfg(target_arch = "wasm32")]
mod web {
    use super::*;
    use anyhow::bail;

    #[instrument(err)]
    pub fn save(data_frame: &DataFrame, name: &str) -> Result<()> {
        let mut workbook = Workbook::new();
        write(data_frame, workbook.add_worksheet())?;
        let buffer = workbook.save_to_buffer()?;
        if let Err(error) = super::super::save(&buffer, name) {
            bail!("save: {error:?}");
        }
        Ok(())
    }
}

fn write(data_frame: &DataFrame, worksheet: &mut Worksheet) -> Result<()> {
    let mut names = vec![];
    // Iterate through the dataframe column by column.
    for (index, series) in data_frame.iter().enumerate() {
        let col = index as u16;

        // Store the column names for use as table headers.
        names.push(series.name().to_string());

        // Write the row data for each column/type.
        for (index, value) in series.iter().enumerate() {
            let row = 1 + index as u32;

            // Map the Polars Series AnyValue types to Excel/rust_xlsxwriter
            // types.
            match value {
                AnyValue::Int8(value) => {
                    worksheet.write_number(row, col, value)?;
                }
                AnyValue::UInt8(value) => {
                    worksheet.write_number(row, col, value)?;
                }
                AnyValue::Int16(value) => {
                    worksheet.write_number(row, col, value)?;
                }
                AnyValue::UInt16(value) => {
                    worksheet.write_number(row, col, value)?;
                }
                AnyValue::Int32(value) => {
                    worksheet.write_number(row, col, value)?;
                }
                AnyValue::UInt32(value) => {
                    worksheet.write_number(row, col, value)?;
                }
                AnyValue::Int64(value) => {
                    worksheet.write_string(row, col, format!("{value:x}"))?;
                }
                AnyValue::UInt64(value) => {
                    worksheet.write_string(row, col, format!("{value:x}"))?;
                }
                AnyValue::Float32(value) => {
                    worksheet.write_number(row, col, value)?;
                }
                AnyValue::Float64(value) => {
                    worksheet.write_number(row, col, value)?;
                }
                AnyValue::String(value) => {
                    worksheet.write_string(row, col, value)?;
                }
                AnyValue::Boolean(value) => {
                    worksheet.write_boolean(row, col, value)?;
                }
                AnyValue::Null => {
                    // Treat Null as blank for now.
                }
                AnyValue::Datetime(value, time_units, _) => {
                    let datetime = match time_units {
                        TimeUnit::Nanoseconds => timestamp_ns_to_datetime(value),
                        TimeUnit::Microseconds => timestamp_us_to_datetime(value),
                        TimeUnit::Milliseconds => timestamp_ms_to_datetime(value),
                    }
                    .expect("invalid or out-of-range datetime");
                    let format = Format::new().set_num_format("yyyy\\-mm\\-dd\\ hh:mm:ss");
                    worksheet.write_datetime_with_format(row, col, &datetime, &format)?;
                    worksheet.set_column_width(col, 18)?;
                }
                _ => {
                    println!(
                        "WARNING: AnyValue data type '{}' is not supported",
                        value.dtype()
                    );
                    break;
                }
            }
        }
    }

    // Create a table for the dataframe range.
    let (rows, cols) = data_frame.shape();
    let mut table = Table::new();
    let columns: Vec<_> = names
        .into_iter()
        .map(|caption| TableColumn::new().set_header(caption))
        .collect();
    table = table.set_columns(&columns);
    // Add the table to the worksheet.
    worksheet.add_table(0, 0, rows as u32, cols as u16 - 1, &table)?;
    // Autofit the columns.
    worksheet.autofit();
    Ok(())
}
