use crate::store::StoreChoice;

use super::{connection, vault::VaultMessage};

#[derive(Debug, Clone)]
pub enum GUIMessage {
    Exit,
    Submit,
    VaultMessage(VaultMessage, String),
    NewVault,
    ShowPassword,
    HidePassword,
    CopyPassword,
    PromptChanged(String),
    PasswordChanged(String),
    ChangeName(String),
    SelectStyle(StoreChoice),
    UpdateField(String, String),
    GeneratePassword,
    CopyClipboard(Option<String>),
    ClearClipboard,
    Event(connection::Event),
    // Send(Message),
}
