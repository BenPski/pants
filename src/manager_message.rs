use crate::message::Message;

#[derive(Debug)]
pub enum ManagerMessage {
    NewVault(String),
    List,
    Info,
    VaultMessage(String, Message),
}
