use boring_derive::From;

use crate::{
    info::Info,
    reads::Reads,
    vault::{dir::BackupFile, entry::Entry, schema::Schema},
};

#[derive(Debug, Clone, From)]
pub enum Output {
    Info(Info),
    Schema(Schema),
    BackupFiles(Vec<BackupFile>),
    Read(Reads<Entry>),
    List(Vec<String>),
    Backup(BackupFile),
    Content(String),
    Nothing,
}
