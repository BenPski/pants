use figment::{Figment, Metadata, Provider};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Config {
    allowed_connections: Vec<Uuid>,
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
        Metadata::named("Vault config")
    }
    fn data(
        &self,
    ) -> Result<figment::value::Map<figment::Profile, figment::value::Dict>, figment::Error> {
        figment::providers::Serialized::defaults(Config::default()).data()
    }
}
