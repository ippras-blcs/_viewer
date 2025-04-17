use crate::{
    app::{metadata::MetaDataFrame, panes::settings::Settings},
    utils::hashed::Hashed,
};
use egui::util::cache::{ComputerMut, FrameCache};
use polars::prelude::*;
use std::{collections::BTreeMap, iter::zip};
use tracing::instrument;

const IDENTIFIER: &str = "Identifier";
const TIMESTAMP: &str = "Timestamp";
const POINTS: &str = "Points";
const X: &str = "X";
const Y: &str = "Y";
const ROUND_DECIMALS: u32 = 6;

/// Plot computed
pub(in crate::app) type Computed = FrameCache<Value, Computer>;

/// Plot computer
#[derive(Default)]
pub(in crate::app) struct Computer;

impl Computer {
    #[instrument(skip(self), err)]
    fn try_compute(&mut self, key: Key) -> PolarsResult<Value> {
        let mut value = Value::default();
        let mut lazy_frame = key.frame.data.clone().lazy();
        lazy_frame = lazy_frame.sort([IDENTIFIER, TIMESTAMP], Default::default());
        // Source
        value.source = source(lazy_frame.clone())?;
        // Resampling
        if key.settings.plot.resampling.mean {
            value.resampling.mean = resampling_mean(lazy_frame.clone(), key)?;
        }
        if key.settings.plot.resampling.median {
            value.resampling.median = resampling_median(lazy_frame.clone(), key)?;
        }
        // Rolling
        if key.settings.plot.rolling.mean {
            value.rolling.mean = rolling_mean(lazy_frame.clone(), key)?;
        }
        if key.settings.plot.rolling.median {
            value.rolling.median = rolling_median(lazy_frame, key)?;
        }
        Ok(value)
    }
}

impl ComputerMut<Key<'_>, Value> for Computer {
    fn compute(&mut self, key: Key) -> Value {
        self.try_compute(key).unwrap()
    }
}

/// Key
#[derive(Clone, Copy, Debug, Hash)]
pub(in crate::app) struct Key<'a> {
    pub(crate) frame: &'a Hashed<MetaDataFrame>,
    pub(crate) settings: &'a Settings,
}

/// Value
#[derive(Clone, Debug, Default)]
pub(in crate::app) struct Value {
    pub(in crate::app) source: BTreeMap<u64, Vec<[f64; 2]>>,
    pub(in crate::app) resampling: Resampling,
    pub(in crate::app) rolling: Rolling,
}

#[derive(Clone, Debug, Default)]
pub(in crate::app) struct Resampling {
    pub(in crate::app) mean: BTreeMap<u64, Vec<[f64; 2]>>,
    pub(in crate::app) median: BTreeMap<u64, Vec<[f64; 2]>>,
}

#[derive(Clone, Debug, Default)]
pub(in crate::app) struct Rolling {
    pub(in crate::app) mean: BTreeMap<u64, Vec<[f64; 2]>>,
    pub(in crate::app) median: BTreeMap<u64, Vec<[f64; 2]>>,
}

fn source(lazy_frame: LazyFrame) -> PolarsResult<BTreeMap<u64, Vec<[f64; 2]>>> {
    collect(
        lazy_frame.group_by([col(IDENTIFIER)]).agg([as_struct(vec![
            col(TIMESTAMP).alias(X),
            last().alias(Y),
        ])
        .alias(POINTS)]),
    )
}

fn resampling_mean(lazy_frame: LazyFrame, key: Key) -> PolarsResult<BTreeMap<u64, Vec<[f64; 2]>>> {
    let every = Duration::parse(&format!("{}s", key.settings.plot.resampling.every));
    let period = Duration::parse(&format!("{}s", key.settings.plot.resampling.period));
    collect(
        lazy_frame
            .group_by_dynamic(
                col(TIMESTAMP),
                [col(IDENTIFIER)],
                DynamicGroupOptions {
                    every,
                    period,
                    offset: Duration::parse("0"),
                    ..Default::default()
                },
            )
            .agg([last().mean()])
            .group_by([col(IDENTIFIER)])
            .agg([as_struct(vec![col(TIMESTAMP).alias(X), last().alias(Y)]).alias(POINTS)]),
    )
}

fn resampling_median(
    lazy_frame: LazyFrame,
    key: Key,
) -> PolarsResult<BTreeMap<u64, Vec<[f64; 2]>>> {
    let every = Duration::parse(&format!("{}s", key.settings.plot.resampling.every));
    let period = Duration::parse(&format!("{}s", key.settings.plot.resampling.period));
    collect(
        lazy_frame
            .group_by_dynamic(
                col(TIMESTAMP),
                [col(IDENTIFIER)],
                DynamicGroupOptions {
                    every,
                    period,
                    offset: Duration::parse("0"),
                    ..Default::default()
                },
            )
            .agg([last().median()])
            .group_by([col(IDENTIFIER)])
            .agg([as_struct(vec![col(TIMESTAMP).alias(X), last().alias(Y)]).alias(POINTS)]),
    )
}

fn rolling_mean(lazy_frame: LazyFrame, key: Key) -> PolarsResult<BTreeMap<u64, Vec<[f64; 2]>>> {
    collect(
        lazy_frame.group_by([col(IDENTIFIER)]).agg([as_struct(vec![
            col(TIMESTAMP).alias(X),
            last()
                .rolling_mean(RollingOptionsFixedWindow {
                    window_size: key.settings.plot.rolling.window_size,
                    min_periods: key.settings.plot.rolling.min_periods,
                    ..Default::default()
                })
                .round(ROUND_DECIMALS)
                .alias(Y),
        ])
        .alias(POINTS)]),
    )
}

fn rolling_median(lazy_frame: LazyFrame, key: Key) -> PolarsResult<BTreeMap<u64, Vec<[f64; 2]>>> {
    collect(
        lazy_frame.group_by([col(IDENTIFIER)]).agg([as_struct(vec![
            col(TIMESTAMP).alias(X),
            last()
                .rolling_median(RollingOptionsFixedWindow {
                    window_size: key.settings.plot.rolling.window_size,
                    min_periods: key.settings.plot.rolling.min_periods,
                    ..Default::default()
                })
                .round(ROUND_DECIMALS)
                .alias(Y),
        ])
        .alias(POINTS)]),
    )
}

fn collect(lazy_frame: LazyFrame) -> PolarsResult<BTreeMap<u64, Vec<[f64; 2]>>> {
    let data_frame = lazy_frame.collect()?;
    let mut value = BTreeMap::new();
    for (identifier, points) in zip(
        data_frame[IDENTIFIER].u64()?.into_no_null_iter(),
        data_frame[POINTS].list()?.into_no_null_iter(),
    ) {
        let x = points.struct_()?.field_by_name(X)?;
        let y = points.struct_()?.field_by_name(Y)?;
        value.insert(
            identifier,
            zip(
                x.cast(&DataType::Float64)?.f64()?.into_no_null_iter(),
                y.cast(&DataType::Float64)?.f64()?.into_no_null_iter(),
            )
            .map(|(x, y)| [x, y])
            .collect(),
        );
    }
    Ok(value)
}
