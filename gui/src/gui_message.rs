use iced::Theme;
use secrecy::Secret;

use pants_store::{store::StoreChoice, Password};

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
    ChangeTheme(Theme),
    Event(connection::Event),
    ClosePopup,
    TabPressed(bool),
    Close,
    Nothing,
    // Send(Message),
}
