use serde::{Deserialize, Serialize};

/// Cache
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub(crate) struct Cache {
    pub(crate) triacylglycerols: Vec<String>,
}

impl Cache {
    pub(crate) fn new() -> Self {
        Self {
            triacylglycerols: Vec::new(),
        }
    }
}
