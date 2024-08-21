use std::{
    fmt::Display,
    fs,
    path::{Path, PathBuf},
};

use boring_derive::From;
use chrono::{DateTime, Utc};
use figment::{
    providers::{Format, Toml},
    Figment,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    errors::{DirError, TimestampFileError},
    utils::{format_date, now, read_date},
};

use super::config::Config;

/// the interface to read the binary data from the various files
#[derive(Debug)]
pub struct VaultDir {
    path: PathBuf,
    id: Uuid,
    config: PathBuf,
    vault: PathBuf,
    schema: PathBuf,
    backups: BackupDir,
}

#[derive(Debug)]
pub struct BackupDir {
    base: PathBuf,
    files: Vec<TimestampedFile>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TimestampedFile {
    path: PathBuf,
    timestamp: DateTime<Utc>,
}

#[derive(Debug, From, Clone)]
pub struct BackupFile(TimestampedFile);

impl From<&TimestampedFile> for BackupFile {
    fn from(value: &TimestampedFile) -> Self {
        Self(value.clone())
    }
}

impl Display for BackupFile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Backup from {}", self.0.timestamp)
    }
}

pub type DirResult<T> = Result<T, DirError>;

impl VaultDir {
    /// load up the directory relevant to a vault
    pub fn new<P: AsRef<Path>>(base: P, id: Uuid) -> DirResult<Self> {
        let path: PathBuf = base.as_ref().into();
        let path = path.join(id.to_string());
        fs::create_dir_all(&path)?;
        let config = path.join("config.toml");
        let vault = path.join("vault.pants");
        let schema = path.join("schema.json");
        let backups = BackupDir::new(path.join("backups"))?;
        Ok(Self {
            path,
            id,
            config,
            vault,
            schema,
            backups,
        })
    }

    pub fn config(&self) -> Figment {
        Config::figment().merge(Toml::file(&self.config))
    }

    /// remove the whole directory
    pub fn remove(&self) -> DirResult<()> {
        let _res = fs::remove_dir_all(&self.path)?;
        Ok(())
    }

    /// write out the vault and it's schema
    pub fn save_vault(&self, vault: &Vec<u8>, schema: &Vec<u8>) -> DirResult<()> {
        self.write_vault(vault)?;
        self.write_schema(schema)?;
        Ok(())
    }

    /// read in the vault from the relevant file
    pub fn read_vault(&self) -> DirResult<Vec<u8>> {
        let content = fs::read(&self.vault)?;
        Ok(content)
    }

    /// write out the vault
    fn write_vault(&self, vault: &Vec<u8>) -> DirResult<()> {
        fs::write(&self.vault, vault)?;
        Ok(())
    }

    /// read in the schema
    pub fn read_schema(&self) -> DirResult<Vec<u8>> {
        let contents = fs::read(&self.schema)?;
        Ok(contents)
    }

    /// write the schema
    fn write_schema(&self, schema: &Vec<u8>) -> DirResult<()> {
        fs::write(&self.schema, schema)?;
        Ok(())
    }

    pub fn create_backup(&self) -> DirResult<()> {
        self.backups.create_backup(&self.vault)
    }

    pub fn swap_backup(&self, backup: TimestampedFile) -> DirResult<()> {
        self.backups.create_backup(&self.vault)?;
        fs::copy(backup, &self.vault)?;
        Ok(())
    }

    pub fn backups(&self) -> impl Iterator<Item = BackupFile> + '_ {
        self.backups.backups()
    }
}

impl BackupDir {
    fn new<P: AsRef<Path>>(base: P) -> DirResult<Self> {
        let base: PathBuf = base.as_ref().into();
        fs::create_dir_all(&base)?;
        let mut files = vec![];
        for entry in base.read_dir()? {
            let entry = entry?;
            // not sure if I actually want to do this or if they should be looked
            // up on demand rather than on load
            let file = TimestampedFile::load(entry.path())?;
            files.push(file);
        }
        Ok(Self { base, files })
    }

    fn create_backup<P: AsRef<Path>>(&self, from: P) -> DirResult<()> {
        let file = TimestampedFile::now(self.base.join("backup.bak"))?;
        fs::copy(from, file)?;
        Ok(())
    }

    fn backups(&self) -> impl Iterator<Item = BackupFile> + '_ {
        self.files.iter().map(|x| BackupFile::from(x))
    }
}

impl TimestampedFile {
    fn load<P: AsRef<Path>>(path: P) -> Result<Self, TimestampFileError> {
        let path: PathBuf = path.as_ref().into();
        if !path.is_file() {
            return Err(TimestampFileError::NotFile(path));
        }
        let name = path
            .file_stem()
            .ok_or_else(|| TimestampFileError::NoFileName(path.clone()))?;
        let split = name
            .to_str()
            .ok_or_else(|| TimestampFileError::ConversionError(path.clone()))?
            .split_once("_")
            .ok_or_else(|| TimestampFileError::NoTimestamp(path.clone()))?;
        let timestamp =
            read_date(split.1).map_err(|e| TimestampFileError::BadTimestamp(path.clone(), e))?;
        Ok(Self { path, timestamp })
    }

    fn new<P: AsRef<Path>>(path: P, timestamp: DateTime<Utc>) -> Result<Self, TimestampFileError> {
        let mut path: PathBuf = path.as_ref().into();
        let mut name = path
            .file_stem()
            .ok_or_else(|| TimestampFileError::NoFileName(path.clone()))?
            .to_str()
            .ok_or_else(|| TimestampFileError::ConversionError(path.clone()))?
            .to_string();
        let ext = if let Some(ext) = path.extension() {
            ext.to_str()
                .ok_or_else(|| TimestampFileError::ConversionError(path.clone()))?
        } else {
            ""
        };
        name.push_str(&format_date(timestamp));
        name.push_str(ext);
        path.set_file_name(name);
        Ok(Self { path, timestamp })
    }

    fn now<P: AsRef<Path>>(path: P) -> Result<Self, TimestampFileError> {
        Self::new(path, now())
    }
}

impl AsRef<Path> for TimestampedFile {
    fn as_ref(&self) -> &Path {
        &self.path
    }
}
