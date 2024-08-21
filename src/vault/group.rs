use std::{
    fs,
    path::{Path, PathBuf},
};

use uuid::Uuid;

use crate::errors::FSError;

use super::dir::VaultDir;

/// the directory that contains all the vaults
#[derive(Debug)]
pub struct VaultGroup {
    path: PathBuf,
}

impl VaultGroup {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<VaultGroup, FSError> {
        fs::create_dir_all(&path).map_err(|e| FSError::Create(e, path.as_ref().into()))?;
        Ok(Self {
            path: path.as_ref().into(),
        })
    }
    pub fn create(&self, id: Uuid) -> Result<VaultDir, FSError> {
        let path = self.path.join(id.to_string());
        if !path.exists() {
            let res = VaultDir::new(path, id)?;
            Ok(res)
        } else {
            Err(FSError::AlreadyExists(id))
        }
    }
    pub fn get(&self, id: Uuid) -> Result<VaultDir, FSError> {
        let path = self.path.join(id.to_string());
        if path.is_dir() {
            let res = VaultDir::new(path, id)?;
            Ok(res)
        } else {
            Err(FSError::NoVault(id))
        }
    }
    pub fn remove(&self, id: Uuid) -> Result<(), FSError> {
        let path = self.path.join(id.to_string());
        fs::remove_dir_all(&path).map_err(|e| FSError::Remove(e, path.into()))?;
        Ok(())
    }
}
