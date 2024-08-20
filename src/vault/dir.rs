use std::path::PathBuf;

use uuid::Uuid;

pub struct VaultDir {
    id: Uuid,
    vault: PathBuf,
    schema: PathBuf,
    backups: BackupDir,
}

struct BackupDir {
    backups: Vec<TimestampedFile>,
}

struct TimestampedFile {}
