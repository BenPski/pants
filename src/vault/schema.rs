use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use super::{entry::Entry, Vault};

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Schema {
    entries: BTreeMap<String, SchemaEntry>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct SchemaEntry {
    values: Vec<String>,
    description: String,
    urls: Vec<String>,
}

impl From<Entry> for SchemaEntry {
    fn from(value: Entry) -> Self {
        Self {
            values: value.values.into_iter().map(|(s, _)| s).collect(),
            description: value.description,
            urls: value.urls,
        }
    }
}

impl From<Vault> for Schema {
    fn from(value: Vault) -> Self {
        Self {
            entries: value
                .entries
                .into_iter()
                .map(|(k, v)| (k, v.into()))
                .collect(),
        }
    }
}
