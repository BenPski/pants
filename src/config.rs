use figment::{
    providers::{Format, Toml},
    value::{Dict, Map},
    Error, Figment, Metadata, Profile, Provider,
};
use pants_gen::password::PasswordSpec;
use serde::{Deserialize, Serialize};

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
    pub fn from<T: Provider>(provider: T) -> Result<Self, Error> {
        Figment::from(provider).extract()
    }

    pub fn figment() -> Figment {
        use figment::providers::Env;
        let mut base_dir = if let Some(project_dirs) =
            directories_next::ProjectDirs::from("com", "bski", "pants")
        {
            project_dirs.data_dir().into()
        } else {
            std::env::current_dir().unwrap_or_default()
        };

        base_dir.push("client.toml");
        Figment::from(ClientConfig::default())
            .merge(Toml::file_exact(base_dir))
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
