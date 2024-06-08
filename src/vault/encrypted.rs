use aes_gcm::{aead::OsRng, Aes256Gcm, Key};
use argon2::password_hash::SaltString;
use serde::{Deserialize, Serialize};

use crate::{
    action::Record,
    secure::{Encrypted, SecureData},
    vault::Vault,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PasswordEncrypted<Data> {
    pub data: Encrypted<Data>,
    pub salt: String,
}

impl<Data> SecureData for PasswordEncrypted<Data> {
    type Item = Data;
    fn salt(&self) -> &str {
        &self.salt
    }
    fn data(&self) -> &Encrypted<Self::Item> {
        &self.data
    }
}

pub type VaultEncrypted = PasswordEncrypted<Vault>;

impl VaultEncrypted {
    pub fn new(password: String) -> anyhow::Result<Self> {
        let salt = SaltString::generate(&mut OsRng).to_string();
        let key = Self::get_key(&salt, password);
        Encrypted::encrypt(&Vault::new(), key).map(|vault| Self { data: vault, salt })
    }

    pub fn from_vault(salt: String, key: Key<Aes256Gcm>, vault: &Vault) -> anyhow::Result<Self> {
        Encrypted::encrypt(vault, key).map(|vault| Self { data: vault, salt })
    }

    pub fn update(&mut self, data: &Vault, key: Key<Aes256Gcm>) -> anyhow::Result<()> {
        let updated = Encrypted::encrypt(data, key)?;
        self.data = updated;
        Ok(())
    }
}

pub type RecordEncrypted = PasswordEncrypted<Record>;

impl RecordEncrypted {
    pub fn new(password: String) -> anyhow::Result<Self> {
        let salt = SaltString::generate(&mut OsRng).to_string();
        let key = Self::get_key(&salt, password);
        Encrypted::encrypt(&Record::new(), key).map(|vault| Self { data: vault, salt })
    }

    pub fn update(&mut self, data: &Record, key: Key<Aes256Gcm>) -> anyhow::Result<()> {
        let updated = Encrypted::encrypt(data, key)?;
        self.data = updated;
        Ok(())
    }
}
