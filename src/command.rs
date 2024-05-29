use std::error::Error;

use crate::{
    cli::{CLICommands, EntryStyle},
    errors::ConversionError,
    schema::Schema,
    store::Store,
};

#[derive(Debug, Clone)]
pub enum Instructions {
    Interaction(Interaction),
    Commands(Commands),
}

#[derive(Debug, Clone)]
pub enum Command {
    Read { key: String },
    Update { key: String, value: Store },
    Delete { key: String },
}

#[derive(Debug, Clone)]
pub enum Interaction {
    List,
    Backup,
    Rotate,
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

impl Instructions {
    fn handle_new(name: String, schema: Schema, style: &str) -> Result<Self, Box<dyn Error>> {
        match schema.get(&name) {
            Some(_) => Err(Box::new(ConversionError::Exists)),
            None => {
                let value = Store::prompt(style)?;
                let commands: Commands = vec![Command::Update { key: name, value }].into();
                Ok(commands.into())
            }
        }
    }
    pub fn from_commands(commands: CLICommands, schema: Schema) -> Result<Self, Box<dyn Error>> {
        match commands {
            CLICommands::New { style } => match style {
                EntryStyle::Password { name } => Self::handle_new(name, schema, "password"),
                EntryStyle::UsernamePassword { name } => {
                    Self::handle_new(name, schema, "username-password")
                }
            },
            CLICommands::Get { key } => {
                let commands: Commands = vec![Command::Read { key }].into();
                Ok(commands.into())
            }
            CLICommands::Update { key } => match schema.get(&key) {
                None => Err(Box::new(ConversionError::NoEntry)),
                Some(value) => {
                    let value = Store::prompt(value)?;
                    let commands: Commands = vec![Command::Update { key, value }].into();
                    Ok(commands.into())
                }
            },
            CLICommands::Delete { key } => {
                let commands: Commands = vec![Command::Delete { key }].into();
                Ok(commands.into())
            }
            CLICommands::List => Ok(Interaction::List.into()),
            CLICommands::Backup => Ok(Interaction::Backup.into()),
            CLICommands::Rotate => Ok(Interaction::Rotate.into()),
        }
    }
}

impl From<Vec<Command>> for Commands {
    fn from(value: Vec<Command>) -> Self {
        Self { commands: value }
    }
}

impl From<Commands> for Instructions {
    fn from(value: Commands) -> Self {
        Instructions::Commands(value)
    }
}

impl From<Interaction> for Instructions {
    fn from(value: Interaction) -> Self {
        Instructions::Interaction(value)
    }
}
