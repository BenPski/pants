use crate::store::StoreChoice;

use super::{connection, entry::EntryMessage};

#[derive(Debug, Clone)]
pub enum GUIMessage {
    Exit,
    Submit,
    EntryMessage(EntryMessage, String),
    ShowPassword,
    HidePassword,
    CopyPassword,
    PasswordChanged(String),
    NewEntry,
    ChangeName(String),
    SelectStyle(StoreChoice),
    UpdateField(String, String),
    GeneratePassword,
    Event(connection::Event),
    // Send(Message),
}
