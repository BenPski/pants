use std::{collections::HashMap, fmt::Display};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Schema {
    pub data: HashMap<String, String>,
}

impl Schema {
    pub fn new(data: HashMap<String, String>) -> Self {
        Self { data }
    }

    pub fn get(&self, key: &str) -> Option<&String> {
        self.data.get(key)
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

impl From<HashMap<String, String>> for Schema {
    fn from(value: HashMap<String, String>) -> Self {
        Self { data: value }
    }
}

impl IntoIterator for Schema {
    type Item = (String, String);
    type IntoIter = std::collections::hash_map::IntoIter<String, String>;

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
