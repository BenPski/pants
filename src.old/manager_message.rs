use crate::{message::Message, Password};

/// the messages to the manager of all the vaults
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
