use crate::{
    command::{Command, Commands},
    store::Store,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Operation {
    Get { key: String },
    Set { key: String, value: Option<Store> },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Operations {
    pub operations: Vec<Operation>,
}

impl Default for Operations {
    fn default() -> Self {
        Self::new()
    }
}

impl Operations {
    pub fn new() -> Self {
        Self { operations: vec![] }
    }

    pub fn push(&mut self, operation: Operation) {
        self.operations.push(operation);
    }

    // pub fn add(mut self, operation: Operation) -> Self {
    //     self.operations.push(operation);
    //     self
    // }
}

impl IntoIterator for Operations {
    type Item = Operation;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.operations.into_iter()
    }
}

impl From<Vec<Operation>> for Operations {
    fn from(value: Vec<Operation>) -> Self {
        Self { operations: value }
    }
}

impl From<Commands> for Operations {
    fn from(commands: Commands) -> Self {
        let mut ops = Operations::new();
        for command in commands {
            match command {
                Command::Read { key } => ops.push(Operation::Get { key }),
                Command::Update { key, value } => {
                    ops.push(Operation::Get { key: key.clone() });
                    ops.push(Operation::Set {
                        key,
                        value: Some(value),
                    });
                }
                Command::Delete { key } => ops.push(Operation::Set { key, value: None }),
            }
        }
        ops
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        command::{Command, Commands},
        operation::{Operation, Operations},
    };

    #[test]
    fn convert_read_to_get() {
        let commands = Commands::from(vec![Command::Read {
            key: "balls".to_string(),
        }]);
        let operations = commands.into();

        assert_eq!(
            Operations::from(vec![Operation::Get {
                key: "balls".to_string()
            }]),
            operations
        );
    }

    // #[test]
    // fn convert_update_to_get_and_set() {
    //     let commands = Commands::from(vec![Command::Update {
    //         key: "balls".to_string(),
    //         value: "weiner".to_string(),
    //     }]);
    //     let operations = commands.into();
    //
    //     assert_eq!(
    //         Operations::from(vec![
    //             Operation::Get {
    //                 key: "balls".to_string()
    //             },
    //             Operation::Set {
    //                 key: "balls".to_string(),
    //                 value: Some("weiner".to_string())
    //             }
    //         ]),
    //         operations
    //     );
    // }

    #[test]
    fn convert_delete_to_set() {
        let commands = Commands::from(vec![Command::Delete {
            key: "balls".to_string(),
        }]);
        let operations = commands.into();

        assert_eq!(
            Operations::from(vec![Operation::Set {
                key: "balls".to_string(),
                value: None
            }]),
            operations
        );
    }
}
