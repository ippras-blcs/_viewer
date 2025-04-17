use polars::prelude::*;
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    hash::{Hash, Hasher},
};

pub const FILE: &str = "File";
pub const ICON: &str = "Icon";
pub const MAX_TIMESTAMP: &str = "MaxTimestamp";
pub const MIN_TIMESTAMP: &str = "MinTimestamp";
pub const NAME: &str = "Name";
// pub const VALUE: &str = "Value";
// pub const MIN_VALUE: &str = "MinValue";
// pub const MAX_VALUE: &str = "MaxValue";
// pub const VERSION: &str = "Version";

/// Metadata
pub type Metadata = BTreeMap<String, String>;

// Metadata frame
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct MetaDataFrame {
    pub(crate) meta: Metadata,
    pub(crate) data: DataFrame,
}

impl MetaDataFrame {
    pub const fn new(meta: Metadata, data: DataFrame) -> Self {
        Self { meta, data }
    }
}

impl Eq for MetaDataFrame {}

impl Hash for MetaDataFrame {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.meta.hash(state);
        assert!(!self.data.should_rechunk());
        for series in self.data.iter() {
            for value in series.iter() {
                value.hash(state);
            }
        }
    }
}

impl PartialEq for MetaDataFrame {
    fn eq(&self, other: &Self) -> bool {
        self.meta == other.meta && self.data.equals_missing(&other.data)
    }
}
