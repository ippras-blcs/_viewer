use serde::{Deserialize, Serialize};

/// Cache
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub(crate) struct Cache {
    pub(crate) identifiers: Vec<String>,
}

impl Cache {
    pub(crate) fn new() -> Self {
        Self {
            identifiers: Vec::new(),
        }
    }
}
