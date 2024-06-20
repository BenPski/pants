use secrecy::Secret;

use crate::{store::StoreChoice, Password};

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
    PasswordChanged(Password),
    PasswordConfirmChanged(Password),
    ChangeName(String),
    SelectStyle(StoreChoice),
    UpdateField(String, Secret<String>),
    GeneratePassword,
    CopyClipboard(Option<Password>),
    ClearClipboard,
    Event(connection::Event),
    // Send(Message),
}
