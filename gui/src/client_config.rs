use figment::{
    value::{Dict, Map},
    Error, Metadata, Profile, Provider,
};
use iced::Theme;
use pants_gen::password::PasswordSpec;
use pants_store::config::internal_config::{BaseConfig, InternalConfig};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ClientConfig {
    pub password_spec: String,
    // seconds
    pub clipboard_time: u64,
    pub theme: String,
}

impl Default for ClientConfig {
    fn default() -> Self {
        Self {
            password_spec: PasswordSpec::default().to_string(),
            clipboard_time: 10,
            theme: Theme::default().to_string(),
        }
    }
}

impl<'de> InternalConfig<'de> for ClientConfig {
    fn name() -> String {
        "gui_client.toml".to_string()
    }
}

impl<'de> BaseConfig<'de> for ClientConfig {}

impl Provider for ClientConfig {
    fn metadata(&self) -> Metadata {
        Metadata::named("Vault config")
    }

    fn data(&self) -> Result<Map<Profile, Dict>, Error> {
        figment::providers::Serialized::defaults(Self::default()).data()
    }
}
