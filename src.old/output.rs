use boring_derive::From;

use crate::{file::BackupFile, info::Info, reads::Reads, schema::Schema, store::Store};

#[derive(Debug, Clone, From)]
pub enum Output {
    Info(Info),
    Schema(Schema),
    BackupFiles(Vec<BackupFile>),
    Read(Reads<Store>),
    List(Vec<String>),
    Backup(BackupFile),
    Content(String),
    Nothing,
}
