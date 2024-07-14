use serde::{Deserialize, Serialize};

use crate::store::Store;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Action {
    // Read {
    //     key: String,
    //     value: Option<String>,
    // },
    Replace {
        key: String,
        start: Option<Store>,
        end: Option<Store>,
    },
    Noop,
}

impl Action {
    pub fn inverse(self) -> Self {
        match self {
            Self::Noop => Self::Noop,
            // Self::Read { key, value } => Self::Read { key, value },
            Self::Replace { key, start, end } => Self::Replace {
                key,
                start: end,
                end: start,
            },
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Record {
    pub actions: Vec<Action>,
}

impl IntoIterator for Record {
    type Item = Action;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.actions.into_iter()
    }
}

impl Default for Record {
    fn default() -> Self {
        Self::new()
    }
}

impl Record {
    pub fn new() -> Self {
        Record { actions: vec![] }
    }

    pub fn push(&mut self, action: Action) {
        self.actions.push(action);
    }

    // pub fn add(mut self, action: Action) -> Self {
    //     self.actions.push(action);
    //     self
    // }
}
