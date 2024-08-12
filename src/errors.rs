use thiserror::Error;

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
