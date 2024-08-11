use std::{collections::BTreeMap, fmt::Display};

use boring_derive::From;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default, From)]
pub struct Schema {
    pub data: BTreeMap<String, Vec<String>>,
}

impl Schema {
    pub fn new() -> Self {
        Self {
            data: BTreeMap::new(),
        }
    }

    pub fn insert(&mut self, key: String, value: Vec<String>) {
        self.data.insert(key, value.into());
    }

    pub fn get(&self, key: &str) -> Option<&Vec<String>> {
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
            .map(|(key, value)| format!("{key}: {value:?}"))
            .collect()
    }
}

impl IntoIterator for Schema {
    type Item = (String, Vec<String>);
    type IntoIter = std::collections::btree_map::IntoIter<String, Vec<String>>;

    fn into_iter(self) -> Self::IntoIter {
        self.data.into_iter()
    }
}

impl Display for Schema {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (key, value) in self.data.iter() {
            writeln!(f, "{key}:")?;
            for field in value {
                writeln!(f, " - {field}")?;
            }
        }
        Ok(())
    }
}
