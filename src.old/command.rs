use crate::store::Store;

#[derive(Debug, Clone)]
pub enum Command {
    Read { key: String },
    Update { key: String, value: Store },
    Delete { key: String },
}

#[derive(Debug, Clone)]
pub struct Commands {
    pub commands: Vec<Command>,
}

impl IntoIterator for Commands {
    type Item = Command;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.commands.into_iter()
    }
}

impl Default for Commands {
    fn default() -> Self {
        Self::new()
    }
}

impl Commands {
    pub fn new() -> Self {
        Self { commands: vec![] }
    }

    pub fn push(&mut self, command: Command) {
        self.commands.push(command);
    }

    // pub fn add(mut self, command: Command) -> Self {
    //     self.commands.push(command);
    //     self
    // }
}

impl From<Vec<Command>> for Commands {
    fn from(value: Vec<Command>) -> Self {
        Self { commands: value }
    }
}

impl From<Command> for Commands {
    fn from(value: Command) -> Self {
        Self {
            commands: vec![value],
        }
    }
}
