use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Schema {
    data: HashMap<String, String>,
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
