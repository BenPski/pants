use std::fmt::Display;

use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct EntryValue(String);
impl From<String> for EntryValue {
    fn from(value: String) -> Self {
        Self(value)
    }
}
impl Display for EntryValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// an entry in the vault
/// at the level of (username, password), security questions, credit card info, etc
#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct Entry {
    pub(crate) values: Vec<(String, EntryValue)>,
    pub(crate) description: String,
    pub(crate) urls: Vec<String>,
}

impl Entry {
    pub fn fields(&self) -> impl Iterator<Item = &String> {
        self.values.iter().map(|(f, _)| f)
    }

    pub fn values(self) -> impl Iterator<Item = (String, EntryValue)> {
        self.values.into_iter()
    }
    pub fn values_iter(&self) -> std::slice::Iter<(String, EntryValue)> {
        self.values.iter()
    }
    pub fn values_mut(&mut self) -> std::slice::IterMut<(String, EntryValue)> {
        self.values.iter_mut()
    }

    pub fn urls(&self) -> std::slice::Iter<String> {
        self.urls.iter()
    }
    pub fn add_url(&mut self, url: String) {
        self.urls.push(url)
    }
    pub fn remove_url(&mut self, url: String) {
        if let Some(pos) = self.urls.iter().position(|x| *x == url) {
            self.urls.swap_remove(pos);
        }
    }

    pub fn description(&self) -> &String {
        &self.description
    }
    pub fn description_mut(&mut self) -> &mut String {
        &mut self.description
    }

    pub fn get(&self, key: &str) -> Option<&EntryValue> {
        for (k, v) in &self.values {
            if k == key {
                return Some(v);
            }
        }
        None
    }
    pub fn get_mut(&mut self, key: &str) -> Option<&mut EntryValue> {
        for (k, v) in self.values.iter_mut() {
            if k == key {
                return Some(v);
            }
        }
        None
    }
    pub fn add_value(&mut self, field: &str, value: EntryValue) {
        if let Some(v) = self.get_mut(field) {
            *v = value;
        } else {
            self.values.push((field.into(), value));
        }
    }
    pub fn remove_value(&mut self, field: &str) -> Option<(String, EntryValue)> {
        let pos = self.values.iter().position(|(k, _)| k == field)?;
        Some(self.values.remove(pos))
    }
    pub fn swap_values(&mut self, first: &str, second: &str) {
        let mut pos1 = 0;
        let mut pos2 = 0;
        for (i, (k, _)) in self.values.iter().enumerate() {
            if k == first {
                pos1 = i;
            }
            if k == second {
                pos2 = i;
            }
        }
        self.values.swap(pos1, pos2);
    }
    pub fn move_values(&mut self, first: &str, second: &str) {
        let mut pos1 = 0;
        let mut pos2 = 0;
        for (i, (k, _)) in self.values.iter().enumerate() {
            if k == first {
                pos1 = i;
            }
            if k == second {
                pos2 = i;
            }
        }
        let temp = self.values[pos1].clone();
        self.values.remove(pos1);
        self.values.insert(pos2, temp);
    }
}
