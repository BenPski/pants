use core::panic;
use std::{process::exit, str::FromStr, thread, time::Duration};

use arboard::Clipboard;
use clap::{Parser, Subcommand, ValueEnum};
use inquire::Confirm;
use pants_gen::password::PasswordSpec;

use crate::{
    config::ClientConfig,
    errors::{ClientError, CommunicationError, SchemaError},
    message::Message,
    output::Output,
    schema::Schema,
    store::Store,
    vault::interface::VaultInterface,
    Password,
};

#[derive(Debug, Clone, Copy, ValueEnum)]
enum OutputStyle {
    Clipboard,
    None,
    Raw,
}

#[derive(Parser)]
pub struct CliArgs {
    #[command(subcommand)]
    pub command: CLICommands,
    /// how to handle values pulled from vault
    #[arg(long, value_enum, default_value_t = OutputStyle::Clipboard)]
    output: OutputStyle,
}

#[derive(Subcommand)]
pub enum CLICommands {
    /// create new entry
    New {
        #[command(subcommand)]
        style: EntryStyle,
        /// specify a password spec string to be used over the configured one
        #[arg(long)]
        spec: Option<String>,
    },
    /// lookup the given entry
    Get { key: String },
    /// update the entry
    Update {
        key: String,
        // #[command(subcommand)]
        // password: Option<Generate>,
        /// specify a password spec string to be used over the configured one
        #[arg(long)]
        spec: Option<String>,
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

pub struct CliApp {
    args: CliArgs,
    config: ClientConfig,
    interface: VaultInterface,
}

impl CliApp {
    pub fn run() {
        let args = CliArgs::parse();
        let config: ClientConfig = ClientConfig::figment().extract().unwrap();
        let interface = VaultInterface::new();
        let app = CliApp {
            args,
            config,
            interface,
        };
        app.execute()
    }

    pub fn execute(self) {
        match self.args.command {
            CLICommands::Gen(args) => {
                if let Some(p) = args.execute() {
                    println!("{p}");
                } else {
                    println!("Could not satisfy password spec constraints");
                }
            }
            ref command => match self.process(command) {
                Ok(()) => (),
                Err(e) => {
                    println!("Encountered error: {}", e);
                    exit(1)
                }
            },
        }
    }
    fn process(&self, command: &CLICommands) -> anyhow::Result<()> {
        let message = self.construct_message(command)?;
        let output = self.interface.receive(message)?;
        self.handle_output(output)
    }
    fn handle_output(&self, output: Output) -> anyhow::Result<()> {
        match output {
            Output::Nothing => Ok(()),
            Output::Read(reads) => {
                if !reads.data.is_empty() {
                    match self.args.output {
                        OutputStyle::Clipboard => {
                            let mut clipboard = Clipboard::new()?;
                            let orig = clipboard.get_text().unwrap_or("".to_string());
                            for (key, value) in reads.data.clone().into_iter() {
                                println!("{}", key);
                                match value {
                                    Store::Password(pass) => {
                                        clipboard.set_text(pass)?;
                                        println!("  password: <Copied to clipboard>");
                                        thread::sleep(Duration::from_secs(
                                            self.config.clipboard_time,
                                        ));
                                    }
                                    Store::UsernamePassword(user, pass) => {
                                        clipboard.set_text(pass)?;
                                        println!("  username: {}", user);
                                        println!("  password: <Copied to clipboard>");
                                        thread::sleep(Duration::from_secs(
                                            self.config.clipboard_time,
                                        ));
                                    }
                                }
                            }
                            clipboard.set_text(orig)?;
                            println!("Resetting clipboard");
                            thread::sleep(Duration::from_secs(1));
                        }
                        OutputStyle::Raw => {
                            println!("{}", reads);
                        }
                        OutputStyle::None => {}
                    }
                } else {
                    println!("Nothing read from vault");
                }
                Ok(())
            }
            Output::List(items) => {
                if items.is_empty() {
                    println!("No entries");
                } else {
                    println!("Available entries:");
                    for item in items {
                        println!("- {}", item);
                    }
                }
                Ok(())
            }
            Output::Schema(schema) => {
                println!("{}", schema);
                Ok(())
            }
            Output::Backup(backup) => {
                println!("Backed up to: {}", backup);
                Ok(())
            }
            Output::BackupFiles(backups) => {
                for file in backups {
                    println!("{}", file);
                }
                Ok(())
            }
        }
    }
    fn construct_message(&self, command: &CLICommands) -> anyhow::Result<Message> {
        match command {
            CLICommands::Get { key } => {
                let password = Self::get_password("Vault password:")?;
                Ok(Message::Get(password, key.to_string()))
            }
            CLICommands::Update { key, spec } => {
                let schema = self.get_schema()?;
                match schema.get(key) {
                    None => Err(Box::new(CommunicationError::NoEntry).into()),
                    Some(style) => {
                        let spec = PasswordSpec::from_str(
                            &spec
                                .clone()
                                .unwrap_or_else(|| self.config.password_spec.clone()),
                        )?;
                        let value = self.prompt(style, spec)?;
                        let password = Self::get_password("Vault password:")?;
                        Ok(Message::Update(password, key.to_string(), value))
                    }
                }
            }
            CLICommands::Delete { key } => {
                let password = Self::get_password("Vault password:")?;
                Ok(Message::Delete(password, key.to_string()))
            }
            CLICommands::New { style, spec } => {
                let schema = self.get_schema()?;
                let spec = PasswordSpec::from_str(
                    &spec.clone().unwrap_or(self.config.password_spec.clone()),
                )?;
                match style {
                    EntryStyle::Password { name } => {
                        self.handle_new(schema, name.to_string(), "password", spec)
                    }
                    EntryStyle::UsernamePassword { name } => {
                        self.handle_new(schema, name.to_string(), "username-password", spec)
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
                    match self.interface.receive(Message::BackupList)? {
                        Output::BackupFiles(files) => {
                            let backup_file = inquire::Select::new("Restore from:", files)
                                .with_help_message("Choose the backup file to restore from")
                                .prompt()?;
                            let password = Self::get_password("Current password")?;
                            let backup_password = Self::get_password("Backup's password:")?;
                            Ok(Message::Restore(password, backup_password, backup_file))
                        }
                        _ => Err(Box::new(CommunicationError::UnexpectedOutput).into()),
                    }
                }
            },
            CLICommands::List => Ok(Message::Schema),
            CLICommands::Gen(_) => panic!("Should have branched before this"),
        }
    }

    fn handle_new(
        &self,
        schema: Schema,
        key: String,
        style: &str,
        spec: PasswordSpec,
    ) -> anyhow::Result<Message> {
        match schema.get(&key) {
            None => {
                let value = self.prompt(style, spec)?;
                let password = Self::get_password("Vault password:")?;
                Ok(Message::Update(password, key, value))
            }
            Some(_) => Err(Box::new(CommunicationError::ExistingEntry).into()),
        }
    }

    fn get_schema(&self) -> anyhow::Result<Schema> {
        match self.interface.receive(Message::Schema)? {
            Output::Schema(schema) => Ok(schema),
            _ => Err(Box::new(CommunicationError::UnexpectedOutput).into()),
        }
    }

    fn get_password(prompt: &str) -> anyhow::Result<Password> {
        let password = inquire::Password::new(prompt)
            .without_confirmation()
            .with_display_toggle_enabled()
            .with_display_mode(inquire::PasswordDisplayMode::Masked)
            .prompt()?;
        Ok(password)
    }

    fn get_password_confirm(prompt: &str) -> anyhow::Result<Password> {
        let password = inquire::Password::new(prompt)
            .with_display_toggle_enabled()
            .with_display_mode(inquire::PasswordDisplayMode::Masked)
            .prompt()?;
        Ok(password)
    }

    fn prompt(&self, repr: &str, spec: PasswordSpec) -> anyhow::Result<Store> {
        match repr {
            "password" => self.get_store_password(spec).map(Store::Password),
            "username-password" => {
                let username_input = inquire::Text::new("Username:")
                    .with_help_message("New username")
                    .prompt();
                let username = username_input?;
                let password = self.get_store_password(spec)?;
                Ok(Store::UsernamePassword(username, password))
            }
            _ => Err(Box::new(SchemaError::BadType).into()),
        }
    }

    fn get_store_password(&self, spec: PasswordSpec) -> anyhow::Result<Password> {
        let generate = Confirm::new("Generate password?")
            .with_default(true)
            .with_help_message("Create a random password or enter manually?")
            .prompt();
        match generate {
            Ok(true) => {
                let password = spec.generate().ok_or(ClientError::BadPasswordSpec)?;
                Ok(password)
            }
            Ok(false) => {
                let password_input = inquire::Password::new("Password: ")
                    .with_display_toggle_enabled()
                    .with_display_mode(inquire::PasswordDisplayMode::Masked)
                    .prompt();
                match password_input {
                    Ok(p) => Ok(p),
                    Err(_) => Err(Box::new(SchemaError::BadValues).into()),
                }
            }
            Err(_) => Err(Box::new(SchemaError::BadValues).into()),
        }
    }
}
