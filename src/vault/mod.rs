pub mod config;
pub mod dir;
pub mod encrypted;
pub mod entry;
pub mod group;
pub mod schema;

use std::collections::BTreeMap;

use entry::Entry;
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Vault {
    entries: BTreeMap<String, Entry>,
}

impl Vault {
    pub fn insert(&mut self, key: &str, entry: Entry) -> Option<Entry> {
        self.entries.insert(key.into(), entry)
    }

    pub fn get(&self, key: &str) -> Option<&Entry> {
        self.entries.get(key)
    }

    pub fn get_mut(&mut self, key: &str) -> Option<&mut Entry> {
        self.entries.get_mut(key)
    }

    pub fn remove(&mut self, key: &str) -> Option<Entry> {
        self.entries.remove(key)
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn keys(self) -> Vec<String> {
        self.entries.into_keys().collect()
    }

    pub fn export(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(&self.entries)
    }
}
