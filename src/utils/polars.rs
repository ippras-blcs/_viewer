use crate::utils::hashed::HashedMetaDataFrame;
use polars::prelude::*;
use polars_utils::format_list_truncated;

pub fn format_list_truncated(frames: &[HashedMetaDataFrame], separator: &str) -> String {
    format_list_truncated!(frames.iter().map(|frame| frame.meta.format(separator)), 2)
}
