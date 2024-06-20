use core::panic;
use std::{process::exit, str::FromStr, thread, time::Duration};

use arboard::Clipboard;
use clap::{Parser, Subcommand, ValueEnum};
use inquire::Confirm;
use pants_gen::password::PasswordSpec;

use crate::{
    config::{client_config::ClientConfig, internal_config::BaseConfig},
    errors::{ClientError, CommunicationError, SchemaError},
    info::Info,
    manager_message::ManagerMessage,
    message::Message,
    output::Output,
    schema::Schema,
    store::Store,
    vault::manager::VaultManager,
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
    /// create new vault
    New { name: String },
    /// create new entry
    Add {
        /// name of the vault
        vault: String,
        #[command(subcommand)]
        style: EntryStyle,
        /// specify a password spec string to be used over the configured one
        #[arg(long)]
        spec: Option<String>,
    },
    /// lookup the given entry
    Get {
        /// name of the vault
        vault: String,
        /// name of the entry
        key: String,
    },
    /// update the entry
    Update {
        /// name of the vault
        vault: String,
        /// name of the entry
        key: String,
        // #[command(subcommand)]
        // password: Option<Generate>,
        /// specify a password spec string to be used over the configured one
        #[arg(long)]
        spec: Option<String>,
    },
    /// delete a vault/entry
    Delete {
        /// name of the vault
        vault: String,
        /// name of the entry
        key: Option<String>,
    },
    /// list the vaults/entries
    List {
        /// name of vault to list entries of
        vault: Option<String>,
    },
    /// interact with backups, defaults to creating a new backup
    Backup {
        /// name of the vault
        vault: String,
        /// how to interact with backups (nothing => make backup, list => list available backups,
        /// restore => copy in a backup)
        #[command(subcommand)]
        option: Option<BackupCommand>,
    },
    /// rotate master password for the vault
    Rotate {
        /// name of the vault
        vault: String,
    }, // Transaction,
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
    interface: VaultManager,
}

impl CliApp {
    pub fn run() {
        let args = CliArgs::parse();
        let config = <ClientConfig as BaseConfig>::load_err();
        let interface = VaultManager::default();
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
            ref command => {
                match Self::process(&self.config, &self.args.output, self.interface, command) {
                    Ok(()) => (),
                    Err(e) => {
                        println!("Encountered error: {}", e);
                        exit(1)
                    }
                }
            }
        }
    }
    fn process(
        config: &ClientConfig,
        output_style: &OutputStyle,
        mut manager: VaultManager,
        command: &CLICommands,
    ) -> anyhow::Result<()> {
        let message = Self::construct_message(&mut manager, config, command)?;
        let output = manager.receive(message)?;
        Self::handle_output(config, output_style, output)
    }
    fn handle_output(
        config: &ClientConfig,
        output_style: &OutputStyle,
        output: Output,
    ) -> anyhow::Result<()> {
        match output {
            Output::Nothing => Ok(()),
            Output::Read(reads) => {
                if !reads.data.is_empty() {
                    match output_style {
                        OutputStyle::Clipboard => {
                            let mut clipboard = Clipboard::new()?;
                            let orig = clipboard.get_text().unwrap_or("".to_string());
                            for (key, value) in reads.data.clone().into_iter() {
                                println!("{}", key);
                                match value {
                                    Store::Password(pass) => {
                                        clipboard.set_text(pass)?;
                                        println!("  password: <Copied to clipboard>");
                                        thread::sleep(Duration::from_secs(config.clipboard_time));
                                    }
                                    Store::UsernamePassword(user, pass) => {
                                        clipboard.set_text(pass)?;
                                        println!("  username: {}", user);
                                        println!("  password: <Copied to clipboard>");
                                        thread::sleep(Duration::from_secs(config.clipboard_time));
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
            Output::Info(data) => {
                if data.data.is_empty() {
                    println!("No vaults created yet");
                } else {
                    for (vault, schema) in data {
                        if schema.is_empty() {
                            println!("{}: no entries", vault);
                        } else {
                            println!("{}:", vault);
                            for (key, value) in schema.data.iter() {
                                println!("  {}: {}", key, value);
                            }
                        }
                    }
                }
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
    fn construct_message(
        manager: &mut VaultManager,
        config: &ClientConfig,
        command: &CLICommands,
    ) -> anyhow::Result<ManagerMessage> {
        match command {
            CLICommands::New { name } => Ok(ManagerMessage::NewVault(name.into())),
            CLICommands::Get { vault, key } => {
                let password = Self::get_password("Vault password:")?;
                Ok(ManagerMessage::VaultMessage(
                    vault.to_string(),
                    Message::Get(password, key.to_string()),
                ))
            }
            CLICommands::Update { vault, key, spec } => {
                let schema = Self::get_schema(manager, vault.into())?;
                match schema.get(key) {
                    None => Err(Box::new(CommunicationError::NoEntry).into()),
                    Some(style) => {
                        let spec = PasswordSpec::from_str(
                            &spec.clone().unwrap_or_else(|| config.password_spec.clone()),
                        )?;
                        let value = Self::prompt(style, spec)?;
                        let password = Self::get_password("Vault password:")?;
                        Ok(ManagerMessage::VaultMessage(
                            vault.into(),
                            Message::Update(password, key.to_string(), value),
                        ))
                    }
                }
            }
            CLICommands::Delete { vault, key } => {
                if let Some(key) = key {
                    let password = Self::get_password("Vault password:")?;
                    Ok(ManagerMessage::VaultMessage(
                        vault.into(),
                        Message::Delete(password, key.to_string()),
                    ))
                } else {
                    let choice =
                        Confirm::new("Are you sure you want to delete the whole vault?").prompt();
                    let schema = Self::get_schema(manager, vault.into())?;
                    if schema.is_empty() {
                        Ok(ManagerMessage::DeleteEmptyVault(vault.into()))
                    } else {
                        let password = Self::get_password("Vault password:")?;
                        match choice {
                            Ok(true) => Ok(ManagerMessage::DeleteVault(vault.into(), password)),
                            _ => Ok(ManagerMessage::Empty),
                        }
                    }
                }
            }
            CLICommands::Add { vault, style, spec } => {
                let info = Self::get_info(manager)?;
                let schema = info.get(vault).cloned().unwrap_or(Schema::default());
                let new_vault = !info.data.contains_key(vault);
                let confirm_password = new_vault || schema.is_empty();
                if new_vault {
                    manager.receive(ManagerMessage::NewVault(vault.into()))?;
                }
                let spec =
                    PasswordSpec::from_str(&spec.clone().unwrap_or(config.password_spec.clone()))?;
                match style {
                    EntryStyle::Password { name } => Self::handle_new(
                        confirm_password,
                        vault.into(),
                        schema,
                        name.to_string(),
                        "password",
                        spec,
                    ),
                    EntryStyle::UsernamePassword { name } => Self::handle_new(
                        confirm_password,
                        vault.into(),
                        schema,
                        name.to_string(),
                        "username-password",
                        spec,
                    ),
                }
            }
            CLICommands::Rotate { vault } => {
                let password = Self::get_password("Vault password:")?;
                let new_password = Self::get_password_confirm("New vault password:")?;
                Ok(ManagerMessage::VaultMessage(
                    vault.into(),
                    Message::Rotate(password, new_password),
                ))
            }
            CLICommands::Backup { vault, option } => match option {
                None => {
                    let password = Self::get_password("Vault password:")?;
                    Ok(ManagerMessage::VaultMessage(
                        vault.into(),
                        Message::Backup(password),
                    ))
                }
                Some(BackupCommand::List) => Ok(ManagerMessage::VaultMessage(
                    vault.into(),
                    Message::BackupList,
                )),
                Some(BackupCommand::Restore) => {
                    match manager.receive(ManagerMessage::VaultMessage(
                        vault.into(),
                        Message::BackupList,
                    ))? {
                        Output::BackupFiles(files) => {
                            let backup_file = inquire::Select::new("Restore from:", files)
                                .with_help_message("Choose the backup file to restore from")
                                .prompt()?;
                            let password = Self::get_password("Current password")?;
                            let backup_password = Self::get_password("Backup's password:")?;
                            Ok(ManagerMessage::VaultMessage(
                                vault.into(),
                                Message::Restore(password, backup_password, backup_file),
                            ))
                        }
                        _ => Err(Box::new(CommunicationError::UnexpectedOutput).into()),
                    }
                }
            },
            // CLICommands::List => Ok(Message::Schema),
            CLICommands::List { vault } => {
                if let Some(name) = vault {
                    Ok(ManagerMessage::VaultMessage(name.into(), Message::Schema))
                } else {
                    Ok(ManagerMessage::Info)
                }
            }
            CLICommands::Gen(_) => panic!("Should have branched before this"),
        }
    }

    fn handle_new(
        new_vault: bool,
        vault: String,
        schema: Schema,
        key: String,
        style: &str,
        spec: PasswordSpec,
    ) -> anyhow::Result<ManagerMessage> {
        match schema.get(&key) {
            None => {
                let value = Self::prompt(style, spec)?;
                let password = if new_vault {
                    Self::get_password_confirm("New vault password:")?
                } else {
                    Self::get_password("Vault password:")?
                };
                Ok(ManagerMessage::VaultMessage(
                    vault,
                    Message::Update(password, key, value),
                ))
            }
            Some(_) => Err(Box::new(CommunicationError::ExistingEntry).into()),
        }
    }

    fn get_schema(manager: &mut VaultManager, vault: String) -> anyhow::Result<Schema> {
        match manager.receive(ManagerMessage::VaultMessage(vault, Message::Schema))? {
            Output::Schema(schema) => Ok(schema),
            _ => Err(Box::new(CommunicationError::UnexpectedOutput).into()),
        }
    }

    fn get_info(manager: &mut VaultManager) -> anyhow::Result<Info> {
        match manager.receive(ManagerMessage::Info)? {
            Output::Info(info) => Ok(info),
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

    fn prompt(repr: &str, spec: PasswordSpec) -> anyhow::Result<Store> {
        match repr {
            "password" => Self::get_store_password(spec).map(Store::Password),
            "username-password" => {
                let username_input = inquire::Text::new("Username:")
                    .with_help_message("New username")
                    .prompt();
                let username = username_input?;
                let password = Self::get_store_password(spec)?;
                Ok(Store::UsernamePassword(username, password))
            }
            _ => Err(Box::new(SchemaError::BadType).into()),
        }
    }

    fn get_store_password(spec: PasswordSpec) -> anyhow::Result<Password> {
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
