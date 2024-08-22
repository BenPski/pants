use std::collections::HashMap;

use figment::{Figment, Metadata, Provider};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// the server config, dictates the owner and the allowed connections
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Config {
    users: HashMap<Uuid, Perm>,
}

/// the level of privledge for a user
/// Read is a bit redundant since right now the presence of a user implies they can
/// read the vault
#[derive(Debug, Default, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub enum Perm {
    #[default]
    Read,
    Write,
    Owner,
}

impl Config {
    /// initial creation of the config, vault must have an owner expected to be the
    /// creator of the vault
    pub fn new(owner: Uuid) -> Self {
        let mut users = HashMap::new();
        users.insert(owner, Perm::Owner);
        Self { users }
    }

    pub fn can_manage(&self, id: &Uuid) -> bool {
        if let Some(perm) = self.users.get(id) {
            perm >= &Perm::Owner
        } else {
            false
        }
    }
    pub fn can_write(&self, id: &Uuid) -> bool {
        if let Some(perm) = self.users.get(id) {
            perm >= &Perm::Write
        } else {
            false
        }
    }
    pub fn can_read(&self, id: &Uuid) -> bool {
        if let Some(perm) = self.users.get(id) {
            perm >= &Perm::Read
        } else {
            false
        }
    }

    fn add_user(&mut self, id: &Uuid, perm: Perm) {
        self.users.insert(*id, perm);
    }
    pub fn remove_user(&mut self, id: &Uuid) {
        self.users.remove(id);
    }
    pub fn get_user(&self, id: &Uuid) -> Option<&Perm> {
        self.users.get(id)
    }
    pub fn get_user_mut(&mut self, id: &Uuid) -> Option<&mut Perm> {
        self.users.get_mut(id)
    }

    pub fn add_owner(&mut self, id: &Uuid) {
        self.users.insert(*id, Perm::Owner);
    }
    pub fn remove_owner(&mut self, id: &Uuid) {
        self.users.remove(id);
    }
    pub fn add_writer(&mut self, id: &Uuid) {
        self.add_user(id, Perm::Write);
    }
    pub fn add_reader(&mut self, id: &Uuid) {
        self.add_user(id, Perm::Read);
    }

    fn levels(&self, perm: Perm) -> impl Iterator<Item = &Uuid> {
        self.users
            .iter()
            .filter_map(move |(id, p)| if p >= &perm { Some(id) } else { None })
    }
    pub fn owners(&self) -> impl Iterator<Item = &Uuid> {
        self.levels(Perm::Owner)
    }
    pub fn writers(&self) -> impl Iterator<Item = &Uuid> {
        self.levels(Perm::Write)
    }
    pub fn readers(&self) -> impl Iterator<Item = &Uuid> {
        self.levels(Perm::Read)
    }

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
