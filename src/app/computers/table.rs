use crate::{
    app::{
        metadata::MetaDataFrame,
        panes::settings::{Order, Settings, Sort},
    },
    utils::hashed::Hashed,
};
use egui::util::cache::{ComputerMut, FrameCache};
use polars::prelude::*;
use tracing::instrument;

/// Table computed
pub(crate) type Computed = FrameCache<Value, Computer>;

/// Table computer
#[derive(Default)]
pub(crate) struct Computer;

impl Computer {
    #[instrument(skip(self), err)]
    fn try_compute(&mut self, key: Key) -> PolarsResult<Value> {
        let mut lazy_frame = key.frame.data.clone().lazy();
        // Filter
        for identifier in &key.settings.table.filter.identifiers {
            lazy_frame = lazy_frame.filter(col("Identifier").neq(lit(*identifier)));
        }
        // Sort
        let mut sort_options = SortMultipleOptions::default();
        if let Order::Descending = key.settings.table.order {
            sort_options = sort_options
                .with_order_descending(true)
                .with_nulls_last(true);
        }
        lazy_frame = match key.settings.table.sort {
            Sort::Identifier => lazy_frame.sort_by_exprs([col("Identifier")], sort_options),
            Sort::Timestamp => lazy_frame.sort_by_exprs([col("Timestamp")], sort_options),
            Sort::Value => lazy_frame.sort_by_exprs([last()], sort_options),
        };
        lazy_frame.collect()
    }
}

impl ComputerMut<Key<'_>, Value> for Computer {
    fn compute(&mut self, key: Key) -> Value {
        self.try_compute(key).unwrap()
    }
}

/// Table key
#[derive(Clone, Copy, Debug, Hash, PartialEq)]
pub(crate) struct Key<'a> {
    pub(crate) frame: &'a Hashed<MetaDataFrame>,
    pub(crate) settings: &'a Settings,
}

/// Table value
type Value = DataFrame;
