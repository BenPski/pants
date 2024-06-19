use crate::{message::Message, Password};

#[derive(Debug)]
pub enum ManagerMessage {
    Empty,
    NewVault(String),
    DeleteVault(String, Password),
    DeleteEmptyVault(String),
    List,
    Info,
    VaultMessage(String, Message),
}
