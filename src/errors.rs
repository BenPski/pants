use std::{error::Error, fmt::Display};

#[derive(Debug)]
pub enum SaveError {
    Format,
    File,
    Write,
}

impl Error for SaveError {}

impl Display for SaveError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Format => write!(f, "Unable to format data"),
            Self::Write => write!(f, "Unable to write to the desired path"),
            Self::File => write!(f, "Unable to create the file or parent directories"),
        }
    }
}

#[derive(Debug)]
pub enum EncryptionError {
    Encryption,
}

impl Error for EncryptionError {}

impl Display for EncryptionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EncryptionError::Encryption => write!(f, "Failed to encrypt data"),
        }
    }
}

#[derive(Debug)]
pub enum DecryptionError {
    Decryption,
}

impl Error for DecryptionError {}

impl Display for DecryptionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DecryptionError::Decryption => write!(f, "Failed to decrypt data"),
        }
    }
}

#[derive(Debug)]
pub enum SchemaError {
    BadType,
    BadValues,
}

// unsafe impl Send for SchemaError {}
// unsafe impl Sync for SchemaError {}

impl Error for SchemaError {}

impl Display for SchemaError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::BadType => write!(f, "Not a valid schema type"),
            Self::BadValues => write!(f, "Invalid values for creating data to store"),
        }
    }
}

#[derive(Debug)]
pub enum CommunicationError {
    UnexpectedOutput,
    NoEntry,
    ExistingEntry,
}

impl Error for CommunicationError {}

impl Display for CommunicationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnexpectedOutput => write!(f, "Received unexpected output from vault"),
            Self::NoEntry => write!(f, "Expected existing entry, but no entry exists"),
            Self::ExistingEntry => write!(f, "Expected no existing entry, but entry exists"),
        }
    }
}
