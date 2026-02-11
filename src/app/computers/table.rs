use crate::{
    app::states::turbidity::settings::{
        Settings,
        table::{Order, Sort},
    },
    utils::hashed::HashedDataFrame,
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
        let mut lazy_frame = key.frame.data_frame.clone().lazy();
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
            Sort::Value => lazy_frame.sort_by_exprs([last().as_expr()], sort_options),
        };
        HashedDataFrame::new(lazy_frame.collect()?)
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
    pub(crate) frame: &'a HashedDataFrame,
    pub(crate) settings: &'a Settings,
}

impl<'a> Key<'a> {
    pub(crate) fn new(frame: &'a HashedDataFrame, settings: &'a Settings) -> Self {
        Self {
            frame,
            settings: &settings,
            // ddof: settings.ddof,
            // normalize_factors: settings.normalize_factors,
            // percent: settings.percent,
            // precision: settings.precision,
            // significant: settings.significant,
            // threshold: &settings.threshold,
        }
    }
}

/// Table value
type Value = HashedDataFrame;
