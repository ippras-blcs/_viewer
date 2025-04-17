use egui::util::hash;
use serde::{Deserialize, Serialize};
use std::{
    hash::{Hash, Hasher},
    ops::Deref,
};

/// Hashed
#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize)]
pub struct Hashed<T> {
    pub value: T,
    pub hash: u64,
}

impl<T: Hash> Hashed<T> {
    pub fn new(value: T) -> Self {
        let hash = hash(&value);
        Self { value, hash }
    }
}

impl<T> Deref for Hashed<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<T> Eq for Hashed<T> {}

impl<T> PartialEq for Hashed<T> {
    fn eq(&self, other: &Self) -> bool {
        self.hash == other.hash
    }
}

impl<T> Hash for Hashed<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.hash.hash(state);
    }
}
