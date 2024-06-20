//! A password manager, as of right now it is command line only
//!
//! The interface works by allowing you to encrypt your data with a master password as most
//! password managers do.
//!
//! On the first creatiion of a vault it will prompt you for a master password to use. If there is
//! a need to rotate the master password the `rotate` command is provided, updating the vaults
//! password to the new master password and creating a backup of the old vault if you need to
//! restore the previous password.
//!
//! Whenever pulling a password out of the vault it will copy it to your clipboard for a few
//! seconds and then attempt to restore the previous contents of your clipboard to prevent
//! unintentional pastes of the password.
//!
//!
//! # Examples
//!
//! The basic interface operates around `new`, `get`, `update`, and `delete`.
//!
//! ## New
//!
//! Creating a new `password` or `username-password` combo.
//! ```bash
//! $ pants new password test
//! > Generate password? Yes
//! > Length of password? 32
//! > Use uppercase letters? Yes
//! > Use lowercase letters? Yes
//! > Use numbers? Yes
//! > Use symbols? Yes
//! > Vault password:  ********
//! test
//!   password: <Copied to clipboard>
//! Resetting clipboard
//! ```
//!
//! ```bash
//! $ pants new username-password check
//! > Username: me
//! > Generate password? No
//! > Password:  ********
//! > Vault password:  ********
//! check
//!   username: me
//!   password: <Copied to clipboard>
//! Resetting clipboard
//! ```
//!
//! ```bash
//! $ pants new password removing
//! > Generate password? Yes
//! > Length of password? 32
//! > Use uppercase letters? Yes
//! > Use lowercase letters? Yes
//! > Use numbers? Yes
//! > Use symbols? Yes
//! > Vault password:  ********
//! removing
//!   password: <Copied to clipboard>
//! Resetting clipboard
//! ```
//!
//! ## Get
//!
//! Retrieve an existing entry
//! ```bash
//! $ pants get test
//! > Vault password:  ********
//! test
//!   password: <Copied to clipboard>
//! Resetting clipboard
//! ```
//!
//! ## Update
//!
//! Update an existing entry.
//! ```bash
//! $ pants update check
//! > Username: mine
//! > Generate password? Yes
//! > Length of password? 32
//! > Use uppercase letters? Yes
//! > Use lowercase letters? Yes
//! > Use numbers? Yes
//! > Use symbols? Yes
//! > Vault password:  ********
//! check
//!   username: mine
//!   password: <Copied to clipboard>
//! Resetting clipboard
//! ```
//!
//! ## Delete
//!
//! Remove an entry
//! ```bash
//! $ pants delete removing
//! > Vault password:  ********
//! Nothing read from vault
//! ```
//!
//! ## List
//!
//! For convenience you can list the existing entries and their type with `list`.
//! ```bash
//! $ pants list
//! Available entries:
//! - check: username-password
//! - test: password//! $ pants list
//! ```
//!
//! # Other commands
//!
//! Other commands include:
//!  - backup: creates a backup of the current vault
//!  - gen: exposes the password generator in [pants-gen](https://docs.rs/pants-gen/)

use secrecy::Secret;
pub mod action;
pub mod cli;
pub mod command;
pub mod config;
pub mod errors;
pub mod file;
pub mod gui;
pub mod info;
pub mod manager_message;
pub mod message;
pub mod operation;
pub mod output;
pub mod reads;
pub mod schema;
pub mod secure;
pub mod store;
pub mod utils;
pub mod vault;

pub type Password = Secret<String>;
