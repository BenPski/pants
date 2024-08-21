use std::{collections::BTreeMap, fmt::Display};

use serde::{Deserialize, Serialize};

use crate::vault::schema::Schema;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Info {
    pub data: BTreeMap<String, Schema>,
}

impl Info {
    pub fn get(&self, key: &str) -> Option<&Schema> {
        self.data.get(key)
    }
    pub fn insert(&mut self, key: String, value: Schema) {
        self.data.insert(key, value);
    }
}

impl From<BTreeMap<String, Schema>> for Info {
    fn from(value: BTreeMap<String, Schema>) -> Self {
        Self { data: value }
    }
}

impl IntoIterator for Info {
    type Item = (String, Schema);
    type IntoIter = std::collections::btree_map::IntoIter<String, Schema>;

    fn into_iter(self) -> Self::IntoIter {
        self.data.into_iter()
    }
}

impl Display for Info {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (vault, schema) in self.data.iter() {
            writeln!(f, "{vault}:")?;
            if schema.is_empty() {
                writeln!(f, "  is empty")?;
            } else {
                for (entry, fields) in &schema.entries {
                    writeln!(f, " {entry}:")?;
                    for field in &fields.values {
                        writeln!(f, "  - {field}")?;
                    }
                    write!(f, "  description {}", fields.description)?;
                    write!(f, "  urls {:?}", fields.urls)?;
                }
            }
        }
        Ok(())
    }
}
