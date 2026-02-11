use metadata::{Metadata, polars::MetaDataFrame};
use polars::prelude::*;
use serde::{Deserialize, Serialize};
use std::{
    borrow::{Borrow, BorrowMut},
    hash::{Hash, Hasher},
    ops::{Deref, DerefMut},
};

/// Hashed meta data frame
pub type HashedMetaDataFrame = MetaDataFrame<Metadata, HashedDataFrame>;

/// Hashed data frame
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct HashedDataFrame {
    #[serde(rename = "bytes")]
    pub data_frame: DataFrame,
    pub hash: u64,
}

impl HashedDataFrame {
    pub const EMPTY: Self = Self {
        data_frame: DataFrame::empty(),
        hash: 0x342948b37d99fce2, // PlSeedableRandomStateQuality::fixed().build_hasher().finish()
    };

    pub fn new(mut data_frame: DataFrame) -> PolarsResult<Self> {
        let hash = hash_data_frame(&mut data_frame)?;
        Ok(Self { data_frame, hash })
    }

    pub fn rehash(&mut self) -> PolarsResult<()> {
        self.hash = hash_data_frame(&mut self.data_frame)?;
        Ok(())
    }
}

impl Borrow<DataFrame> for HashedDataFrame {
    fn borrow(&self) -> &DataFrame {
        &self.data_frame
    }
}

impl BorrowMut<DataFrame> for HashedDataFrame {
    fn borrow_mut(&mut self) -> &mut DataFrame {
        &mut self.data_frame
    }
}

impl Deref for HashedDataFrame {
    type Target = DataFrame;

    fn deref(&self) -> &Self::Target {
        &self.data_frame
    }
}

impl DerefMut for HashedDataFrame {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.data_frame
    }
}

impl Eq for HashedDataFrame {}

impl PartialEq for HashedDataFrame {
    fn eq(&self, other: &Self) -> bool {
        self.hash == other.hash
    }
}

impl Hash for HashedDataFrame {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.hash.hash(state);
    }
}

pub fn hash_data_frame(data_frame: &mut DataFrame) -> PolarsResult<u64> {
    Ok(data_frame
        .with_row_index(PlSmallStr::EMPTY, None)?
        .hash_rows(Some(PlSeedableRandomStateQuality::fixed()))?
        .xor_reduce()
        .unwrap_or_default())
}
