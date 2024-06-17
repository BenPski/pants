use crate::{
    config::{self, internal_config::InternalConfig, manager_config::ManagerConfig},
    errors::ManagerError,
    manager_message::ManagerMessage,
    output::Output,
    utils,
};

use super::interface::VaultInterface;

pub struct VaultManager {
    config: ManagerConfig,
}

impl Default for VaultManager {
    fn default() -> Self {
        Self {
            config: ManagerConfig::load_err(),
        }
    }
}

impl VaultManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn receive(&mut self, message: ManagerMessage) -> anyhow::Result<Output> {
        match message {
            ManagerMessage::NewVault(name) => {
                if self.config.map.contains_key(&name) {
                    Err(ManagerError::VaultExists.into())
                } else {
                    let mut path = utils::base_path();
                    path.push(name.clone());
                    self.config.map.insert(name, path.to_str().unwrap().into());
                    self.config.save();
                    Ok(().into())
                }
            }
            ManagerMessage::VaultMessage(name, message) => {
                if let Some(path) = self.config.map.get(&name) {
                    let interface = VaultInterface::new(path.to_path_buf());
                    interface.receive(message)
                } else {
                    Err(ManagerError::VaultDoesNotExist.into())
                }
            }
            ManagerMessage::List => Ok(self
                .config
                .map
                .clone()
                .into_keys()
                .collect::<Vec<_>>()
                .into()),
        }
    }
}
