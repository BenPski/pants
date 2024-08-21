use argon2::Argon2;
use encrypt_stuff::symmetric::KeySizeUser;
use encrypt_stuff::{
    serialization::{bitcode::Bitcode, decode::Decoder},
    symmetric::encryption::{Encrypted, Encryption},
    DefaultScheme,
};
use secrecy::ExposeSecret;
use serde::{Deserialize, Serialize};

use crate::{errors::EncryptionErrors, DefaultKey, Password};

use super::Vault;

/// represents the vault that is password protected
/// it is what is read from and to the server
#[derive(Debug, Serialize, Deserialize)]
pub struct PasswordProtected {
    data: Encrypted<Vault>,
    salt: Vec<u8>,
}

impl PasswordProtected {
    pub fn load(bytes: &[u8]) -> Result<PasswordProtected, EncryptionErrors> {
        Bitcode::decode(bytes).map_err(|e| EncryptionErrors::Serialization(e.to_string()))
    }

    pub fn key(&self, password: Password) -> Result<Box<DefaultKey>, EncryptionErrors> {
        let mut out = vec![0u8; <<DefaultScheme as Encryption>::Cipher as KeySizeUser>::key_size()];
        Argon2::default()
            .hash_password_into(password.expose_secret().as_bytes(), &self.salt, &mut out)
            .map_err(|e| EncryptionErrors::Key(e))?;
        let key = DefaultKey::from_slice(&out);
        Ok(Box::new(*key))
    }

    pub fn unlock(self, key: DefaultKey) -> Result<UnlockedVault, EncryptionErrors> {
        let decrypted = DefaultScheme::decrypt_exposed(&self.data, &key)
            .map_err(|e| EncryptionErrors::Decryption(e.to_string()))?;
        let decoded = DefaultScheme::extract_exposed(&decrypted)
            .map_err(|e| EncryptionErrors::Deserialization(e.to_string()))?;
        Ok(UnlockedVault {
            data: decoded,
            salt: self.salt,
            key,
        })
    }
}

/// the unlocked vault with the information needed for re-encrypting the vault again
pub struct UnlockedVault {
    data: Vault,
    salt: Vec<u8>,
    key: DefaultKey,
}

impl UnlockedVault {
    pub fn lock(self) -> Result<PasswordProtected, EncryptionErrors> {
        let encrypted = DefaultScheme::encrypt(&self.data, &self.key)
            .map_err(|e| EncryptionErrors::Encryption(e.to_string()))?;
        Ok(PasswordProtected {
            data: encrypted,
            salt: self.salt,
        })
    }
}
