use crate::store::StoreChoice;

use super::{connection, vault::VaultMessage};

#[derive(Debug, Clone)]
pub enum GUIMessage {
    Exit,
    Submit,
    // EntryMessage(EntryMessage, String),
    VaultMessage(VaultMessage, String),
    ShowPassword,
    HidePassword,
    CopyPassword,
    PasswordChanged(String),
    // NewEntry,
    ChangeName(String),
    SelectStyle(StoreChoice),
    UpdateField(String, String),
    GeneratePassword,
    CopyClipboard(Option<String>),
    ClearClipboard,
    Event(connection::Event),
    // Send(Message),
}
