use std::collections::HashMap;

use crate::{
    file::BackupFile,
    store::{Changes, Store},
    Password,
};

/// messages that are used to send to a particular vault
#[derive(Debug, Clone)]
pub enum Message {
    Get(Password, String),
    Update(Password, String, Store),
    Change(Password, String, Changes),
    Delete(Password, String),
    Backup(Password),
    Rotate(Password, Password),
    Restore(Password, Password, BackupFile),
    Rename(Password, String, String),
    Export(Password),
    Import(Password, HashMap<String, Store>),
    Schema,
    BackupList,
}
