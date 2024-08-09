use std::{collections::BTreeMap, fmt::Display};

use serde::{Deserialize, Serialize};

use crate::store::StoreType;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Schema {
    pub data: BTreeMap<String, StoreType>,
}

impl Schema {
    pub fn new() -> Self {
        Self {
            data: BTreeMap::new(),
        }
    }

    pub fn insert(&mut self, key: String, value: StoreType) {
        self.data.insert(key, value);
    }

    pub fn get(&self, key: &str) -> Option<&StoreType> {
        self.data.get(key)
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    pub fn keys(self) -> Vec<String> {
        self.data.into_keys().collect()
    }

    pub fn all_info(self) -> Vec<String> {
        self.data
            .into_iter()
            .map(|(key, value)| format!("{}: {}", key, value))
            .collect()
    }
}

impl From<BTreeMap<String, StoreType>> for Schema {
    fn from(value: BTreeMap<String, StoreType>) -> Self {
        Self { data: value }
    }
}

impl IntoIterator for Schema {
    type Item = (String, StoreType);
    type IntoIter = std::collections::btree_map::IntoIter<String, StoreType>;

    fn into_iter(self) -> Self::IntoIter {
        self.data.into_iter()
    }
}

impl Display for Schema {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (key, value) in self.data.iter() {
            writeln!(f, "{}: {}", key, value)?;
        }
        Ok(())
    }
}
