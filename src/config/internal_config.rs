use std::{fs, path::PathBuf};

use figment::{
    providers::{Format, Toml},
    Error, Figment, Provider,
};
use serde::{Deserialize, Serialize};

use crate::utils;

pub trait InternalConfig<'de>
where
    Self: Default + Provider + Serialize + Deserialize<'de>,
{
    fn name() -> String;
    fn path() -> PathBuf {
        let mut base_dir = utils::base_path();
        base_dir.push(Self::name());
        base_dir
    }
    fn from<T: Provider>(provider: T) -> Result<Self, Error> {
        Figment::from(provider).extract()
    }

    // NOTE: not sure if this is the best place to be specifying the config file since it is possible
    // that the client actually wants to replace or choose the config file location
    fn figment() -> Figment {
        use figment::providers::Env;

        Figment::from(Self::default())
            .merge(Toml::file_exact(Self::path()))
            .merge(Env::prefixed("PANTS_"))
    }

    fn create() -> anyhow::Result<Self> {
        let path = Self::path();
        let config = Self::default();
        if let Some(dir) = path.parent() {
            fs::create_dir_all(dir)?;
        }

        fs::write(path, toml::to_string(&config)?)?;
        Ok(config)
    }

    fn load() -> anyhow::Result<Self> {
        match Self::figment().extract() {
            Ok(c) => Ok(c),
            Err(_) => Self::create(),
        }
    }

    fn load_err() -> Self {
        Self::load().unwrap_or_else(|_| {
            panic!(
                "Unable to load or create '{:?}', please create manually",
                Self::path()
            )
        })
    }
}