pub mod encrypted;
pub mod interface;
pub mod manager;

use core::str;
use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::{
    action::{Action, Record},
    command::Commands,
    operation::{Operation, Operations},
    reads::Reads,
    schema::Schema,
    store::Store,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vault {
    data: BTreeMap<String, Store>,
}

impl Default for Vault {
    fn default() -> Self {
        Self::new()
    }
}

impl Vault {
    pub fn new() -> Vault {
        Self {
            data: BTreeMap::new(),
        }
    }

    fn step(&self, reads: &mut Reads<Store>, operation: Operation) -> Action {
        match operation {
            Operation::Get { key } => {
                if let Some(value) = self.data.get(&key) {
                    reads.insert(key, value.clone());
                }
                Action::Noop
            }
            Operation::Set { key, value } => {
                let previous = match value {
                    None => reads.remove(&key),
                    Some(ref v) => reads.insert(key.clone(), v.clone()),
                };
                Action::Replace {
                    key,
                    start: previous,
                    end: value,
                }
            }
        }
    }

    fn operate(&mut self, operations: Operations) -> (Reads<Store>, Record) {
        let mut record = Record::new();
        let mut reads = Reads::new();
        for operation in operations {
            let action = self.step(&mut reads, operation);
            record.push(action);
        }

        (reads, record)
    }

    pub fn transaction(&mut self, commands: Commands) -> (Reads<Store>, Record) {
        self.operate(commands.into())
    }

    fn apply_action(&mut self, action: Action) {
        if let Action::Replace { key, start: _, end } = action {
            match end {
                Some(value) => {
                    self.data.insert(key, value);
                }
                None => {
                    self.data.remove(&key);
                }
            }
        }
    }

    pub fn apply_record(&mut self, record: Record) {
        for action in record {
            self.apply_action(action)
        }
    }

    pub fn keys(self) -> Vec<String> {
        self.data.into_keys().collect()
    }

    pub fn schema(&self) -> Schema {
        let mut schema = Schema::new();
        for (key, value) in &self.data {
            schema.insert(
                key.to_string(),
                value.data.iter().map(|(x, _)| x.to_string()).collect(),
            );
        }
        schema
    }

    pub fn export(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(&self.data)
    }
}
