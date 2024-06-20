use crate::{
    config::{
        internal_config::{BaseConfig, InternalConfig},
        manager_config::ManagerConfig,
    },
    errors::ManagerError,
    info::Info,
    manager_message::ManagerMessage,
    message::Message,
    output::Output,
    utils,
};

use super::interface::VaultInterface;

pub struct VaultManager {
    config: ManagerConfig,
}

impl Default for VaultManager {
    fn default() -> Self {
        let config = <ManagerConfig as BaseConfig>::load_err();

        Self { config }
    }
}

impl VaultManager {
    pub fn new() -> anyhow::Result<Self> {
        <ManagerConfig as BaseConfig>::load().map(|config| Self { config })
    }

    pub fn receive(&mut self, message: ManagerMessage) -> anyhow::Result<Output> {
        match message {
            ManagerMessage::Empty => Ok(().into()),
            ManagerMessage::NewVault(name) => {
                if let std::collections::btree_map::Entry::Vacant(e) =
                    self.config.map.entry(name.clone())
                {
                    let mut path = utils::base_path();
                    path.push(name.clone());
                    e.insert(path.to_str().unwrap().into());
                    self.config.save()?;
                    Ok(().into())
                } else {
                    Err(ManagerError::VaultExists.into())
                }
            }
            ManagerMessage::DeleteVault(name, password) => {
                if let Some(path) = self.config.map.get(&name) {
                    let interface = VaultInterface::new(path.to_path_buf());
                    interface.delete(password)?;
                    self.config.map.remove(&name);
                    self.config.save()?;
                    Ok(().into())
                } else {
                    Err(ManagerError::VaultDoesNotExist.into())
                }
            }
            ManagerMessage::DeleteEmptyVault(name) => {
                if let Some(path) = self.config.map.get(&name) {
                    let interface = VaultInterface::new(path.to_path_buf());
                    interface.delete_empty()?;
                    self.config.map.remove(&name);
                    self.config.save()?;
                    Ok(().into())
                } else {
                    Err(ManagerError::VaultDoesNotExist.into())
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
            ManagerMessage::Info => {
                let mut info = Info::default();
                for (name, path) in &self.config.map {
                    let interface = VaultInterface::new(path.to_path_buf());
                    if let Ok(Output::Schema(schema)) = interface.receive(Message::Schema) {
                        info.insert(name.to_string(), schema);
                    }
                }
                Ok(info.into())
            }
        }
    }
}
