use std::{error::Error, process::exit};

use clap::{Parser, Subcommand};

use crate::{
    errors::CommunicationError, message::Message, output::Output, schema::Schema, store::Store,
    vault::interface::VaultInterface, Password,
};

#[derive(Parser)]
pub struct CliArgs {
    #[command(subcommand)]
    pub command: CLICommands,
}

#[derive(Subcommand)]
pub enum CLICommands {
    /// create new entry
    New {
        #[command(subcommand)]
        style: EntryStyle,
    },
    /// lookup the given entry
    Get { key: String },
    /// update the entry
    Update {
        key: String,
        // #[command(subcommand)]
        // password: Option<Generate>,
    },
    /// delete the entry
    Delete { key: String },
    /// list the entries in the vault
    List,
    /// interact with backups, defaults to creating a new backup
    Backup {
        #[command(subcommand)]
        option: Option<BackupCommand>,
    },
    /// rotate master password for the vault
    Rotate, // Transaction,
    /// generate password
    Gen(pants_gen::cli::CliArgs),
}

#[derive(Subcommand)]
pub enum EntryStyle {
    Password {
        name: String,
        // #[command(subcommand)]
        // password: Option<Generate>,
    },
    UsernamePassword {
        name: String,
    },
}

#[derive(Subcommand)]
pub enum BackupCommand {
    /// list available backups
    List,
    /// restore from existing backups
    Restore,
}

impl CliArgs {
    pub fn run() {
        let args = CliArgs::parse();
        args.execute()
    }

    pub fn execute(self) {
        match self.command {
            CLICommands::Gen(args) => {
                if let Some(p) = args.execute() {
                    println!("{p}");
                } else {
                    println!("Could not satisfy password spec constraints");
                }
            }

            command => match Self::construct_message(command) {
                Ok(message) => match VaultInterface::receive(message) {
                    Ok(output) => {
                        if let Err(e) = output.finish() {
                            println!("Encountered error: {}", e);
                        }
                    }
                    Err(e) => {
                        println!("Encountered error: {}", e);
                    }
                },
                Err(e) => {
                    println!("Encountered error: {}", e);
                    exit(1)
                }
            },
        }
    }

    fn construct_message(command: CLICommands) -> Result<Message, Box<dyn Error>> {
        match command {
            CLICommands::Get { key } => {
                let password = Self::get_password("Vault password:")?;
                Ok(Message::Get(password, key))
            }
            CLICommands::Update { key } => {
                let schema = Self::get_schema()?;
                match schema.get(&key) {
                    None => Err(Box::new(CommunicationError::NoEntry)),
                    Some(style) => {
                        let value = Store::prompt(style)?;
                        let password = Self::get_password("Vault password:")?;
                        Ok(Message::Update(password, key, value))
                    }
                }
            }
            CLICommands::Delete { key } => {
                let password = Self::get_password("Vault password:")?;
                Ok(Message::Delete(password, key))
            }
            CLICommands::New { style } => {
                let schema = Self::get_schema()?;
                match style {
                    EntryStyle::Password { name } => Self::handle_new(schema, name, "password"),
                    EntryStyle::UsernamePassword { name } => {
                        Self::handle_new(schema, name, "username-password")
                    }
                }
            }
            CLICommands::Rotate => {
                let password = Self::get_password("Vault password:")?;
                let new_password = Self::get_password_confirm("New vault password:")?;
                Ok(Message::Rotate(password, new_password))
            }
            CLICommands::Backup { option } => match option {
                None => {
                    let password = Self::get_password("Vault password:")?;
                    Ok(Message::Backup(password))
                }
                Some(BackupCommand::List) => Ok(Message::BackupList),
                Some(BackupCommand::Restore) => {
                    match VaultInterface::receive(Message::BackupList)? {
                        Output::BackupFiles(files) => {
                            let backup_file = inquire::Select::new("Restore from:", files)
                                .with_help_message("Choose the backup file to restore from")
                                .prompt()?;
                            let password = Self::get_password("Current password")?;
                            let backup_password = Self::get_password("Backup's password:")?;
                            Ok(Message::Restore(password, backup_password, backup_file))
                        }
                        _ => Err(Box::new(CommunicationError::UnexpectedOutput)),
                    }
                }
            },
            CLICommands::List => Ok(Message::Schema),

            _ => todo!(),
        }
    }

    fn handle_new(schema: Schema, key: String, style: &str) -> Result<Message, Box<dyn Error>> {
        match schema.get(&key) {
            None => {
                let value = Store::prompt(style)?;
                let password = Self::get_password("Vault password:")?;
                Ok(Message::Update(password, key, value))
            }
            Some(_) => Err(Box::new(CommunicationError::ExistingEntry)),
        }
    }

    fn get_schema() -> Result<Schema, Box<dyn Error>> {
        match VaultInterface::receive(Message::Schema)? {
            Output::Schema(schema) => Ok(schema),
            _ => Err(Box::new(CommunicationError::UnexpectedOutput)),
        }
    }

    fn get_password(prompt: &str) -> Result<Password, Box<dyn Error>> {
        let password = inquire::Password::new(prompt)
            .without_confirmation()
            .with_display_toggle_enabled()
            .with_display_mode(inquire::PasswordDisplayMode::Masked)
            .prompt()?;
        Ok(password)
    }

    fn get_password_confirm(prompt: &str) -> Result<Password, Box<dyn Error>> {
        let password = inquire::Password::new(prompt)
            .with_display_toggle_enabled()
            .with_display_mode(inquire::PasswordDisplayMode::Masked)
            .prompt()?;
        Ok(password)
    }
}
