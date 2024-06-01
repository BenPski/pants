use std::path::PathBuf;

use crate::{store::Store, Password};

// messages that are used to send to the server
pub enum Message {
    Get(Password, String),
    Update(Password, String, Store),
    Delete(Password, String),
    Backup(Password),
    Rotate(Password, Password),
    Restore(Password, Password, PathBuf),
    Schema,
    BackupList,
}
