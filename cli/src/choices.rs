use std::fmt::Display;

use enum_iterator::Sequence;

/// What to do when creating a new entry
#[derive(Debug, Clone, Copy, Sequence)]
pub enum NewEntry {
    /// generate an entry based off password spec
    Generated,
    /// enter a value manually
    Manual,
    /// stop processing
    Done,
}

impl Display for NewEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Generated => write!(f, "Randomly generated"),
            Self::Manual => write!(f, "Manually entered"),
            Self::Done => write!(f, "Done"),
        }
    }
}

/// What to do when updating an entry
#[derive(Debug, Clone, Sequence)]
pub enum UpdateEntry {
    /// randomly generate the value
    Generate,
    /// manually enter the value
    Manual,
    /// delete the entry
    Delete,
    /// swap current entry with another one
    Swap,
    /// move the entry to be at the position of another
    Move,
    /// do nothing
    Cancel,
}

impl Display for UpdateEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Delete => write!(f, "Remove entry"),
            Self::Generate => write!(f, "Randomize value"),
            Self::Manual => write!(f, "Manually enter value"),
            Self::Swap => write!(f, "Swap position with another entry"),
            Self::Move => write!(f, "Move position to another entry"),
            Self::Cancel => write!(f, "Cancel"),
        }
    }
}

/// choosing what fields to update
#[derive(Debug, Clone)]
pub enum FieldChoice {
    /// an existing field
    Existing(String),
    /// a new field
    New,
    /// done editing entry
    Done,
}

impl Display for FieldChoice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Existing(s) => write!(f, "{s}"),
            Self::New => write!(f, "New entry"),
            Self::Done => write!(f, "Done"),
        }
    }
}
