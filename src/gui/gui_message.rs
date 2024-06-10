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
    CopyClipboard(Option<String>),
    ClearClipboard,
    Event(connection::Event),
    // Send(Message),
}
