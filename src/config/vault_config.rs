use std::path::PathBuf;

use figment::{
    value::{Dict, Map},
    Error, Metadata, Profile, Provider,
};
use serde::{Deserialize, Serialize};

use crate::{file::SaveDir, utils};

use super::internal_config::InternalConfig;

#[derive(Debug, Serialize, Deserialize)]
pub struct VaultConfig {
    save_dir: PathBuf,
}

impl Default for VaultConfig {
    fn default() -> Self {
        let save_dir = utils::base_path();
        Self { save_dir }
    }
}

impl VaultConfig {
    pub fn save_dir(&self) -> SaveDir {
        SaveDir::new(self.save_dir.to_path_buf())
    }
}

impl<'de> InternalConfig<'de> for VaultConfig {
    fn name() -> String {
        "vault.toml".into()
    }
}

impl Provider for VaultConfig {
    fn metadata(&self) -> Metadata {
        Metadata::named("Vault config")
    }

    fn data(&self) -> Result<Map<Profile, Dict>, Error> {
        figment::providers::Serialized::defaults(Self::default()).data()
    }
}
