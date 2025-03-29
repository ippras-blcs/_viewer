use crate::app::panes::pane::table::Settings;
use egui::{
    emath::OrderedFloat,
    util::cache::{ComputerMut, FrameCache},
};
use polars::prelude::*;
use std::hash::{Hash, Hasher};

// const ROUND_DECIMALS: u32 = 6;

/// Table computed
pub(in crate::app) type Computed = FrameCache<Value, Computer>;

/// Table computer
#[derive(Default)]
pub(in crate::app) struct Computer;

impl ComputerMut<Key<'_>, Value> for Computer {
    fn compute(&mut self, key: Key) -> Value {
        let mut lazy_frame = key
            .data_frame
            .clone()
            .lazy()
            .sort(["Time"], Default::default());
        let every = Duration::parse(&format!("{}s", key.downsampling.every));
        let period = Duration::parse(&format!("{}s", key.downsampling.period));
        lazy_frame = lazy_frame
            .group_by_dynamic(
                col("Time"),
                [],
                DynamicGroupOptions {
                    every,
                    period,
                    offset: Duration::parse("0"),
                    ..Default::default()
                },
            )
            .agg([nth(1).median().round(ROUND_DECIMALS)]);
        // .agg([col(name)
        //     .sort_by([col(name).abs()], Default::default())
        //     .last()]);
        lazy_frame.collect().unwrap()
    }
}

/// Key
#[derive(Clone, Copy, Debug)]
pub(in crate::app) struct Key<'a> {
    pub(in crate::app) data_frame: &'a DataFrame,
    pub(in crate::app) settings: &'a Settings,
}

impl Hash for Key<'_> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        if let Ok(names) = self.data_frame[0].str() {
            for name in names {
                name.hash(state);
            }
        }
        if let Ok(values) = self.data_frame[1].f64() {
            for value in values {
                value.map(OrderedFloat).hash(state);
            }
        }
        self.settings.hash(state);
    }
}

/// Value
type Value = DataFrame;
