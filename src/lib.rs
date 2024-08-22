//! A password manager can use the CLI or the GUI.
//!
//! The interface works by allowing you to encrypt your data with a master password as most (all?)
//! password managers do.
//!
//! Pants allows you to create multiple vaults each with a different master password. Pants
//! attempts to keep your master password in memory for as little time as possible, the result is
//! that you'll be prompted to enter your password whenever a transaction with the vault requires
//! encrypting or decrypting data (aka it doesn't cache your password the first time you open the
//! program or individual vault). This is only noticeable in the GUI. If there is
//! a need to rotate the master password the `rotate` command is provided, updating the vaults
//! password to the new master password and creating a backup of the old vault if you need to
//! restore the previous password.
//!
//! Whenever pulling a password out of the vault it will copy it to your clipboard for a few
//! seconds and then attempt to restore the previous contents of your clipboard to prevent
//! unintentional pastes of the password.
//!
//! If you need to change the default behavior like the clipboard time or the default password
//! specification, check the `pants/*_client.toml` located in the standard config directory for
//! your OS (e.g. ~/.local/share/ on linux)
//!
//! # Examples
//!
//! The basic interface operates around `new`, `add`, `get`, `update`, and `delete`.
//!
//! ## New
//!
//! Creating a new vault
//! ```bash
//! $ pants new test
//! ```
//!
//! ## Add
//!
//! Creating a new `password` or `username-password` combo.
//! ```bash
//! # pants add test password example
//! > Generate password? Yes
//! > Vault password: ********
//! example
//!  password: <Copied to clipboard>
//! Resetting clipboard
//! ```
//!
//! ## Get
//!
//! Retrieve an existing entry
//! ```bash
//!$ pants get test test
//! > Vault password: ********
//! test
//!   password: <Copied to clipboard>
//! Resetting clipboard
//! ```
//!
//! ## Update
//!
//! Update an existing entry.
//! ```bash
//! $ pants update test test
//! > Generate password? Yes
//! > Vault password: ********
//! test
//!   password: <Copied to clipboard>
//! Resetting clipboard
//! ```
//!
//! ## Delete
//!
//! Remove a vault
//! $ pants delete example
//! > Are you sure you want to delete the whole vault? Yes
//!
//! Remove an entry
//! ```bash
//! $ pants delete test example
//! > Vault password: ********
//! ```
//!
//! ## List
//!
//! For convenience you can list the existing entries and their type with `list`.
//! ```bash
//! $ pants list
//! something: no entries
//! test:
//!   blah: password
//!   other: password
//!   something: password
//!   test: password
//! ```
//!
//! # Other commands
//!
//! Other commands include:
//!  - backup: creates a backup of the current vault
//!  - gen: exposes the password generator in [pants-gen](https://docs.rs/pants-gen/)

use aes_gcm::Key;
use encrypt_stuff::{symmetric::encryption::Encryption, DefaultScheme};
use secrecy::Secret;
// pub mod config;
pub mod errors;
pub mod info;
// pub mod manager_message;
// pub mod message;
pub mod client;
pub mod output;
pub mod reads;
pub mod utils;
pub mod vault;

pub type Password = Secret<String>;
pub type DefaultKey = Key<<DefaultScheme as Encryption>::Cipher>;
