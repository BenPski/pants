use crate::message::Message;

#[derive(Debug)]
pub enum ManagerMessage {
    NewVault(String),
    List,
    VaultMessage(String, Message),
}
