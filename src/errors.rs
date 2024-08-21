use std::path::PathBuf;

use boring_derive::From;
use thiserror::Error;
use uuid::Uuid;

#[derive(Error, Debug)]
pub enum SaveError {
    #[error("Unable to format data")]
    Format,
    #[error("Unable to create the file or the parent directories")]
    File,
    #[error("Unable to write to the desired path")]
    Write,
}

#[derive(Error, Debug)]
pub enum EncryptionError {
    #[error("Failed to encrypt data")]
    Encryption,
}

#[derive(Error, Debug)]
pub enum DecryptionError {
    #[error("Failed to decrypt data")]
    Decryption,
}

#[derive(Error, Debug)]
pub enum SchemaError {
    #[error("Not a valid schema type")]
    BadType,
    #[error("Invalid values for creating data to store")]
    BadValues,
}

#[derive(Error, Debug)]
pub enum CommunicationError {
    #[error("Received unexpected output from the vault")]
    UnexpectedOutput,
    #[error("Expected an existing entry, but no entry exists")]
    NoEntry,
    #[error("Expected no existing entry, but an entry exists")]
    ExistingEntry,
}

#[derive(Debug, Error)]
pub enum ClientError {
    #[error("Contraints for the password spec could not be met")]
    BadPasswordSpec,
    #[error("Not creating a new vault")]
    NotCreatingVault,
    #[error("Expected to read a value, but got nothing")]
    ReadNothing,
    #[error("No changes made")]
    NoChanges,
    #[error("Couldn't rename the entry")]
    CantRename,
}

#[derive(Debug, Error)]
pub enum ManagerError {
    #[error("Vault already exists")]
    VaultExists,
    #[error("Vault does not exist")]
    VaultDoesNotExist,
    #[error("Tried to delete a non-empty vault")]
    NonEmptyVault,
}

#[derive(Debug, Error, From)]
pub enum FSError {
    #[from(skip)]
    #[error("Vault with Uuid {0} doesn't exist")]
    NoVault(Uuid),
    #[error("{0}")]
    Vault(DirError),
    #[from(skip)]
    #[error("Vault with Uuid {0} already exists")]
    AlreadyExists(Uuid),
    #[from(skip)]
    #[error("Failed to create directory {1}, `{0}`")]
    Create(std::io::Error, PathBuf),
    #[from(skip)]
    #[error("Failed to create directory {1}, `{0}`")]
    Remove(std::io::Error, PathBuf),
}

#[derive(Debug, Error)]
pub enum EncryptionErrors {
    #[error("Failed serialization `{0}`")]
    Serialization(String),
    #[error("Failed encryption `{0}`")]
    Encryption(String),
    #[error("Failed deserialization `{0}`")]
    Deserialization(String),
    #[error("Failed decryption `{0}`")]
    Decryption(String),
    #[error("{0}")]
    Key(argon2::Error),
}

#[derive(Debug, Error, From)]
pub enum DirError {
    #[from(skip)]
    #[error("Failed serialization process `{0}`")]
    Serialization(String),
    #[from(skip)]
    #[error("Failed encryption process `{0}`")]
    Encryption(String),
    #[error("{0}")]
    IO(std::io::Error),
    #[error("{0}")]
    Timestamped(TimestampFileError),
}

#[derive(Debug, Error)]
pub enum TimestampFileError {
    #[error("Unable to convert to utf-8 string `{0}`")]
    ConversionError(PathBuf),
    #[error("Timestamped file has no file name `{0}`")]
    NoFileName(PathBuf),
    #[error("Path is not a file `{0}`")]
    NotFile(PathBuf),
    #[error("Timestamped file has no timestamp `{0}`")]
    NoTimestamp(PathBuf),
    #[error("Failed to parse timestamp for `{0}`: `{1}`")]
    BadTimestamp(PathBuf, chrono::ParseError),
}
