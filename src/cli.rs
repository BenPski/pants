use clap::{Args, Parser, Subcommand};

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
    /// create a backup of the vault
    Backup,
    /// rotate master password for the vault
    Rotate, // Transaction,
}

#[derive(Subcommand)]
pub enum Generate {
    /// create a randomly generated password
    Generate(PasswordSpec),
}

#[derive(Args, Clone)]
pub struct PasswordSpec {
    /// length of the password to generate
    #[arg(short, long, default_value_t = 32)]
    pub length: usize,
    /// minimum number of uppercase characters to include
    #[arg(short, long, default_value_t = 1)]
    pub upper: usize,
    /// minimum number of lowercase characters to include
    #[arg(short = 'd', long, default_value_t = 1)]
    pub lower: usize,
    /// minimum number of numbers to include
    #[arg(short, long, default_value_t = 1)]
    pub numbers: usize,
    /// minimum number of symbols to include
    #[arg(short, long, default_value_t = 1)]
    pub symbols: usize,
}

#[derive(Args)]
struct OutputSpec {
    /// print the results to terminal
    #[arg(long)]
    pub print: bool,
    /// copy result to clipboard
    #[arg(long)]
    pub clipboard: bool,
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
