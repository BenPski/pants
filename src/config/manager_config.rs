use std::{collections::BTreeMap, path::PathBuf};

use figment::{
    value::{Dict, Map},
    Error, Metadata, Profile, Provider,
};
use serde::{Deserialize, Serialize};

use crate::config::internal_config::InternalConfig;

use super::internal_config::BaseConfig;

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct ManagerConfig {
    pub map: BTreeMap<String, PathBuf>,
}

impl<'de> InternalConfig<'de> for ManagerConfig {
    fn name() -> String {
        "pants.toml".into()
    }
}

impl<'de> BaseConfig<'de> for ManagerConfig {}

impl Provider for ManagerConfig {
    fn metadata(&self) -> Metadata {
        Metadata::named("Pants config")
    }

    fn data(&self) -> Result<Map<Profile, Dict>, Error> {
        figment::providers::Serialized::defaults(Self::default()).data()
    }
}
