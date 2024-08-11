use std::{fs, path::PathBuf, process::exit, str::FromStr};

use clap::{CommandFactory, Parser, Subcommand};
use clap_complete::{generate, Shell};
use pants_gen::password::PasswordSpec;

use enum_iterator::all;
use pants_store::{
    config::internal_config::BaseConfig,
    errors::{ClientError, CommunicationError},
    info::Info,
    manager_message::ManagerMessage,
    message::Message,
    output::Output,
    schema::Schema,
    store::{Changes, SecretValue, Store, StoredValue},
    vault::manager::VaultManager,
    Password,
};

use crate::{
    choices::{FieldChoice, NewEntry, UpdateEntry},
    client_config::ClientConfig,
    gen_args,
    output::OutputStyle,
};

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
        /// the name of the entry to add
        key: String,
        /// name of the vault
        #[arg(default_value = "default")]
        vault: String,
        /// specify a password spec string to be used over the configured one
        #[arg(long)]
        spec: Option<String>,
    },
    /// lookup the given entry
    Get {
        /// name of the entry
        key: String,
        /// name of the vault
        #[arg(default_value = "default")]
        vault: String,
    },
    /// update the entry
    Update {
        /// name of the entry
        key: String,
        /// name of the vault
        #[arg(default_value = "default")]
        vault: String,
        // #[command(subcommand)]
        // password: Option<Generate>,
        /// specify a password spec string to be used over the configured one
        #[arg(long)]
        spec: Option<String>,
    },
    /// delete a vault/entry
    Delete {
        /// name of the entry
        key: Option<String>,
        /// name of the vault
        #[arg(default_value = "default")]
        vault: String,
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
                        println!("Error: {}", e);
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
        _config: &ClientConfig,
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
                        println!(" - {}", item);
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
                    println!("{data}");
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
                    Some(fields) => {
                        let spec = PasswordSpec::from_str(
                            &spec.clone().unwrap_or_else(|| config.password_spec.clone()),
                        )?;
                        let changes = Self::prompt_update(&fields, &spec)?;
                        let password = Self::get_password("Vault password:")?;
                        Ok(ManagerMessage::VaultMessage(
                            vault.into(),
                            Message::Change(password, key.to_string(), changes),
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
                        inquire::Confirm::new("Are you sure you want to delete the whole vault?")
                            .prompt();
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
            CLICommands::Add { key, vault, spec } => {
                let spec =
                    PasswordSpec::from_str(&spec.clone().unwrap_or(config.password_spec.clone()))?;

                let password = Self::password_prompt_add(manager, vault)?;
                let store = Self::prompt_add(&spec)?;
                Ok(ManagerMessage::VaultMessage(
                    vault.into(),
                    Message::Update(password, key.into(), store),
                ))
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
                let info = Self::get_info(manager)?;
                let schema = info.get(vault).cloned().unwrap_or(Schema::default());
                let new_vault = !info.data.contains_key(vault);
                let confirm_password = new_vault || schema.is_empty();
                if new_vault {
                    manager.receive(ManagerMessage::NewVault(vault.into()))?;
                }
                let content = fs::read_to_string(path)?;
                let data = serde_json::from_str(&content)?;
                let password = if confirm_password {
                    Self::get_password_confirm("Vault password:")?
                } else {
                    Self::get_password("Vault password:")?
                };
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

    /// prompt for a password and handle the case that the user needs to create
    /// a new vault
    fn password_prompt_add(manager: &mut VaultManager, vault: &str) -> anyhow::Result<Password> {
        let info = Self::get_info(manager)?;
        let schema = info.get(vault).cloned().unwrap_or(Schema::default());
        let new_vault = !info.data.contains_key(vault);
        let confirm_password = new_vault || schema.is_empty();
        if new_vault {
            manager.receive(ManagerMessage::NewVault(vault.into()))?;
        }
        if confirm_password {
            let ans =
                inquire::Confirm::new(&format!("Do you want to create new vault: `{vault}`?"))
                    .prompt()?;
            if ans {
                Self::get_password_confirm(&format!("Password for {vault}:"))
            } else {
                Err(ClientError::NotCreatingVault.into())
            }
        } else {
            Self::get_password(&format!("Password for {vault}:"))
        }
    }

    /// Prompt for a new entry into a vault
    fn prompt_add(spec: &PasswordSpec) -> anyhow::Result<Store> {
        let mut store = Store::default();
        loop {
            match Self::prompt_new_entry(spec)? {
                None => break,
                Some((k, v)) => store.insert(&k, v),
            }
        }
        Ok(store)
    }

    fn prompt_new_entry(spec: &PasswordSpec) -> anyhow::Result<Option<(String, SecretValue)>> {
        let choice = inquire::Select::new("Type of entry", all::<NewEntry>().collect()).prompt()?;
        match choice {
            NewEntry::Done => Ok(None),
            NewEntry::Generated => {
                let ident_input = inquire::Text::new("Name of field:")
                    .with_help_message("The type of the field (username, password, etc)")
                    .prompt()?;
                let value = spec.generate().ok_or(ClientError::BadPasswordSpec)?;
                Ok(Some((ident_input, StoredValue::new(value).into())))
            }
            NewEntry::Manual => {
                let ident_input = inquire::Text::new("Name of field:")
                    .with_help_message("The type of the field (username, password, etc)")
                    .prompt()?;
                let value_input = inquire::Text::new("Value for field:")
                    .with_help_message("The actual value to be stored.")
                    .prompt()?;
                Ok(Some((ident_input, StoredValue::new(value_input).into())))
            }
        }
    }

    fn prompt_update(orig: &[String], spec: &PasswordSpec) -> anyhow::Result<Changes> {
        let mut changes = Changes::new(orig);
        loop {
            let mut fields: Vec<FieldChoice> = changes
                .fields()
                .into_iter()
                .map(|s| FieldChoice::Existing(s))
                .collect();
            fields.push(FieldChoice::New);
            fields.push(FieldChoice::Done);
            let field_choice = inquire::Select::new("Field to update:", fields).prompt()?;
            match field_choice {
                FieldChoice::Done => break,
                FieldChoice::New => {
                    if let Some((k, v)) = Self::prompt_new_entry(spec)? {
                        changes.insert(&k, v);
                    }
                }
                FieldChoice::Existing(s) => {
                    let choice = inquire::Select::new(
                        &format!("How to update {s}:"),
                        all::<UpdateEntry>().collect(),
                    )
                    .prompt()?;
                    match choice {
                        UpdateEntry::Cancel => {}
                        UpdateEntry::Delete => {
                            changes.remove(&s);
                        }
                        UpdateEntry::Manual => {
                            let value_input = inquire::Text::new("New value:").prompt()?;
                            changes.insert(&s, StoredValue::new(value_input).into());
                        }
                        UpdateEntry::Generate => {
                            let value = spec.generate().ok_or(ClientError::BadPasswordSpec)?;
                            changes.insert(&s, StoredValue::new(value).into())
                        }
                        UpdateEntry::Swap => {
                            let choices =
                                changes.fields().into_iter().filter(|k| *k != s).collect();
                            let value = inquire::Select::new("Swap with:", choices).prompt()?;
                            changes.swap(&s, &value);
                        }
                    }
                }
            }
        }

        Ok(changes)
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
}
