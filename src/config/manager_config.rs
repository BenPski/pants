use std::{collections::HashMap, path::PathBuf};

use figment::{
    value::{Dict, Map},
    Error, Metadata, Profile, Provider,
};
use serde::{Deserialize, Serialize};

use crate::config::internal_config::InternalConfig;

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct ManagerConfig {
    pub map: HashMap<String, PathBuf>,
}

impl<'de> InternalConfig<'de> for ManagerConfig {
    fn name() -> String {
        "pants.toml".into()
    }
}

impl Provider for ManagerConfig {
    fn metadata(&self) -> Metadata {
        Metadata::named("Pants config")
    }

    fn data(&self) -> Result<Map<Profile, Dict>, Error> {
        figment::providers::Serialized::defaults(Self::default()).data()
    }
}
