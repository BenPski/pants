use crate::{file::BackupFile, store::Store, Password};

// messages that are used to send to the server
#[derive(Debug, Clone)]
pub enum Message {
    Get(Password, String),
    Update(Password, String, Store),
    Delete(Password, String),
    Backup(Password),
    Rotate(Password, Password),
    Restore(Password, Password, BackupFile),
    Schema,
    BackupList,
}
