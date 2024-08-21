use std::{collections::BTreeMap, fmt::Display};

use serde::{Deserialize, Serialize};

use super::{entry::Entry, Vault};

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct Schema {
    pub(crate) entries: BTreeMap<String, SchemaEntry>,
}

impl Display for Schema {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (key, value) in self.entries.iter() {
            writeln!(f, "{key}:")?;
            writeln!(f, "  description: {}", value.description)?;
            writeln!(f, "  urls: {:?}", value.urls)?;
            for field in &value.values {
                writeln!(f, " - {field}")?;
            }
        }
        Ok(())
    }
}
#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct SchemaEntry {
    pub(crate) values: Vec<String>,
    pub(crate) description: String,
    pub(crate) urls: Vec<String>,
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

impl Schema {
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}
