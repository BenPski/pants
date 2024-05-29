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
pub enum ConversionError {
    PasswordGeneration,
    NoEntry,
    Exists,
    PromptError,
}

impl Error for ConversionError {}

impl Display for ConversionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::PasswordGeneration => {
                write!(f, "Constraints for password generation could not be met")
            }
            Self::NoEntry => write!(f, "No relevant entry found in schema"),
            Self::Exists => write!(f, "Entry already exists"),
            Self::PromptError => write!(f, "Error during prompting"),
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
pub enum InteractionError {
    DifferentPasswords,
}

impl Error for InteractionError {}

impl Display for InteractionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DifferentPasswords => write!(f, "Given passwords don't match"),
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
