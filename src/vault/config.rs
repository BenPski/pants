use std::path::PathBuf;

use figment::{
    providers::{Format, Toml},
    value::{Dict, Map},
    Error, Figment, Metadata, Profile, Provider,
};
use serde::{Deserialize, Serialize};

use crate::{file::SaveDir, utils};

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    save_dir: PathBuf,
}

impl Default for Config {
    fn default() -> Self {
        let save_dir = utils::base_path();
        Config { save_dir }
    }
}

impl Config {
    // NOTE: not sure if this is the best place to be specifying the config file since it is possible
    // that the client actually wants to replace or choose the config file location
    fn path() -> PathBuf {
        let mut base_dir = utils::base_path();
        base_dir.push("vault.toml");
        base_dir
    }
    pub fn from<T: Provider>(provider: T) -> Result<Self, Error> {
        Figment::from(provider).extract()
    }

    pub fn figment() -> Figment {
        use figment::providers::Env;
        Figment::from(Config::default())
            .merge(Toml::file_exact(Self::path()))
            .merge(Env::prefixed("PANTS_"))
    }

    pub fn save_dir(&self) -> SaveDir {
        SaveDir::new(self.save_dir.to_path_buf())
    }
}

impl Provider for Config {
    fn metadata(&self) -> Metadata {
        Metadata::named("Vault config")
    }

    fn data(&self) -> Result<Map<Profile, Dict>, Error> {
        figment::providers::Serialized::defaults(Config::default()).data()
    }
}
