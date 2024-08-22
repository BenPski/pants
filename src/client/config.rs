use figment::{Figment, Metadata, Provider};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// client config, stores it's own id and the vaults it knows about
#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    id: Uuid,
    vaults: Vec<Connection>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            id: Uuid::new_v4(),
            vaults: Vec::new(),
        }
    }
}

/// general description of a vault's location
/// a vault can be on a server or local and always has a uuid
#[derive(Debug, Serialize, Deserialize)]
pub struct Connection {
    location: Location,
    id: Uuid,
}

impl Connection {
    pub fn local(id: Uuid) -> Self {
        Self {
            location: Location::Local,
            id,
        }
    }
    pub fn remote(location: impl Into<String>, id: Uuid) -> Self {
        Self {
            location: Location::Remote(location.into()),
            id,
        }
    }
}

/// a location for the vault storage not sure if this a necessary thing yet, but
/// maybe it will be
#[derive(Debug, Serialize, Deserialize)]
pub enum Location {
    /// some server location (an ip address, url, etc)
    Remote(String),
    /// runs on the local device so gets a shortcut, maybe
    Local,
}

impl Config {
    pub fn from<T: Provider>(provider: T) -> Result<Config, figment::Error> {
        Figment::from(provider).extract()
    }
    pub fn figment() -> Figment {
        Figment::from(Config::default())
    }
}

impl Provider for Config {
    fn metadata(&self) -> figment::Metadata {
        Metadata::named("Client config")
    }
    fn data(
        &self,
    ) -> Result<figment::value::Map<figment::Profile, figment::value::Dict>, figment::Error> {
        figment::providers::Serialized::defaults(Config::default()).data()
    }
}
