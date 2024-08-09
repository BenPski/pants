use std::{fs, path::PathBuf, process::exit, str::FromStr, thread, time::Duration};

use arboard::Clipboard;
use clap::{CommandFactory, Parser, Subcommand, ValueEnum};
use clap_complete::{generate, Shell};
use inquire::{Confirm};
use pants_gen::password::PasswordSpec;

use pants_store::{
    config::internal_config::BaseConfig,
    errors::{ClientError, CommunicationError, SchemaError},
    info::Info,
    manager_message::ManagerMessage,
    message::Message,
    output::Output,
    reads::Reads,
    schema::Schema,
    store::{SecretValue, Store, StoreType, StoredValue},
    vault::manager::VaultManager,
    Password,
};
use secrecy::ExposeSecret;

use crate::{client_config::ClientConfig, gen_args};

#[derive(Debug, Clone, Copy, ValueEnum)]
enum OutputStyle {
    Clipboard,
    None,
    Raw,
}

impl OutputStyle {
    fn handle_reads(&self, reads: Reads<Store>) -> anyhow::Result<()> {
        if reads.data.is_empty() {
            println!("No data read from vault");
            return Ok(());
        }
        match self {
            OutputStyle::Clipboard => {
                let mut clipboard = Clipboard::new()?;
                let orig = clipboard.get_text().unwrap_or("".to_string());
                for (key, value) in reads.data.clone().into_iter() {
                    println!("{key}");
                    for (ident, item) in value.data {
                        clipboard.set_text(item.expose_secret().to_string())?;

                        if let Err(_) = inquire::Text::new(&format!(
                            "Copied `{ident}` to clipboard, hit enter to continue"
                        ))
                        .prompt()
                        {
                            break;
                        }
                    }
                }
                clipboard.set_text(orig)?;
                println!("Resetting clipboard");
                thread::sleep(Duration::from_secs(1));
            }
            OutputStyle::Raw => {
                println!("{:?}", reads);
            }
            OutputStyle::None => {}
        }
        Ok(())
    }
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
    },
    /// export the contents of the vault
    Export {
        /// name of the vault
        vault: String,
    },
    /// import a file into a vault
    Import {
        /// name of the vault
        vault: String,
        /// path to the file
        path: PathBuf,
    },
    /// generate password
    Gen(gen_args::CliArgs),
    /// generate completion file
    Completion { shell: Shell },
}

#[derive(Subcommand)]
pub enum EntryStyle {
    /// dealing with a password alone
    Password {
        name: String,
        // #[command(subcommand)]
        // password: Option<Generate>,
    },
    /// dealing with username and password
    UsernamePassword { name: String },
    /// the entry is some generic data
    Generic { name: String },
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
            CLICommands::Completion { shell } => {
                generate(
                    shell,
                    &mut CliArgs::command(),
                    "pants",
                    &mut std::io::stdout(),
                );
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
                output_style.handle_reads(reads)?;
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
            Output::Content(s) => {
                println!("{s}");
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
                        &StoreType::Password,
                        spec,
                    ),
                    EntryStyle::UsernamePassword { name } => Self::handle_new(
                        confirm_password,
                        vault.into(),
                        schema,
                        name.to_string(),
                        &StoreType::UsernamePassword,
                        spec,
                    ),
                    EntryStyle::Generic { name } => Self::handle_new(
                        confirm_password,
                        vault.into(),
                        schema,
                        name.to_string(),
                        &StoreType::Generic,
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
            CLICommands::Export { vault } => {
                let password = Self::get_password("Vault password:")?;
                Ok(ManagerMessage::VaultMessage(
                    vault.into(),
                    Message::Export(password),
                ))
            }
            CLICommands::Import { vault, path } => {
                println!("Readign file");
                let content = fs::read_to_string(path)?;
                println!("parsing file");
                let data = serde_json::from_str(&content)?;
                let password = Self::get_password("Vault password:")?;
                Ok(ManagerMessage::VaultMessage(
                    vault.into(),
                    Message::Import(password, data),
                ))
            }
            CLICommands::Gen(_) | CLICommands::Completion { .. } => {
                panic!("Should have branched before this")
            }
        }
    }

    fn handle_new(
        new_vault: bool,
        vault: String,
        schema: Schema,
        key: String,
        style: &StoreType,
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
        Ok(password.into())
    }

    fn get_password_confirm(prompt: &str) -> anyhow::Result<Password> {
        let password = inquire::Password::new(prompt)
            .with_display_toggle_enabled()
            .with_display_mode(inquire::PasswordDisplayMode::Masked)
            .prompt()?;
        Ok(password.into())
    }

    fn prompt(ty: &StoreType, spec: PasswordSpec) -> anyhow::Result<Store> {
        match ty {
            StoreType::Password => Self::get_store_password(spec).map(Store::password),
            StoreType::UsernamePassword => {
                let username_input = inquire::Text::new("Username:")
                    .with_help_message("New username")
                    .prompt();
                let username = username_input?;
                let password = Self::get_store_password(spec)?;
                Ok(Store::username_password(
                    StoredValue::new(username),
                    password,
                ))
            }
            StoreType::Generic => {
                let mut vals = Vec::new();
                loop {
                    let ident_input = inquire::Text::new("Name:")
                        .with_help_message(
                            "What is the item being stored (e.g. password, username, etc), enter nothing to finish",
                        )
                        .prompt()?;
                    if ident_input.is_empty() {
                        break;
                    }
                    let value_input = inquire::Text::new("Value:")
                        .with_help_message("The value to store")
                        .prompt()?;
                    vals.push((ident_input, StoredValue::new(value_input).into()));
                }

                Ok(Store::new(StoreType::Generic, vals))
            }
        }
    }

    fn get_store_password(spec: PasswordSpec) -> anyhow::Result<SecretValue> {
        let generate = Confirm::new("Generate password?")
            .with_default(true)
            .with_help_message("Create a random password or enter manually?")
            .prompt();
        match generate {
            Ok(true) => {
                let password = spec.generate().ok_or(ClientError::BadPasswordSpec)?;
                Ok(StoredValue::new(password).into())
            }
            Ok(false) => {
                let password_input = inquire::Password::new("Password: ")
                    .with_display_toggle_enabled()
                    .with_display_mode(inquire::PasswordDisplayMode::Masked)
                    .prompt();
                match password_input {
                    Ok(p) => Ok(StoredValue::new(p).into()),
                    Err(_) => Err(Box::new(SchemaError::BadValues).into()),
                }
            }
            Err(_) => Err(Box::new(SchemaError::BadValues).into()),
        }
    }
}
