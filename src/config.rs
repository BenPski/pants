use std::path::PathBuf;

use figment::{
    providers::{Format, Toml},
    value::{Dict, Map},
    Error, Figment, Metadata, Profile, Provider,
};
use pants_gen::password::PasswordSpec;
use serde::{Deserialize, Serialize};

use crate::utils;

#[derive(Debug, Serialize, Deserialize)]
pub struct ClientConfig {
    pub password_spec: String,
    // seconds
    pub clipboard_time: u64,
}

impl Default for ClientConfig {
    fn default() -> Self {
        ClientConfig {
            password_spec: PasswordSpec::default().to_string(),
            clipboard_time: 10,
        }
    }
}

impl ClientConfig {
    // NOTE: not sure if this is the best place to be specifying the config file since it is possible
    // that the client actually wants to replace or choose the config file location
    fn path() -> PathBuf {
        let mut base_dir = utils::base_path();
        base_dir.push("client.toml");
        base_dir
    }
    pub fn from<T: Provider>(provider: T) -> Result<Self, Error> {
        Figment::from(provider).extract()
    }

    pub fn figment() -> Figment {
        use figment::providers::Env;

        Figment::from(ClientConfig::default())
            .merge(Toml::file_exact(Self::path()))
            .merge(Env::prefixed("PANTS_"))
    }
}

impl Provider for ClientConfig {
    fn metadata(&self) -> Metadata {
        Metadata::named("Vault config")
    }

    fn data(&self) -> Result<Map<Profile, Dict>, Error> {
        figment::providers::Serialized::defaults(ClientConfig::default()).data()
    }
}
