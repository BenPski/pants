use clap::{Parser, Subcommand};

#[derive(Parser)]
pub struct CliArgs {
    #[command(subcommand)]
    pub command: Option<CLICommands>,
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
