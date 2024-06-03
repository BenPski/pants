use std::marker::PhantomData;

use aes_gcm::{
    aead::{generic_array::GenericArray, Aead, OsRng},
    AeadCore, Aes256Gcm, Key, KeyInit,
};
use argon2::{password_hash::SaltString, Argon2};
use serde::{Deserialize, Serialize};

use crate::errors::{DecryptionError, EncryptionError};

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct Encrypted<Data> {
    nonce: Vec<u8>,
    data: Vec<u8>,
    #[serde(skip)]
    data_type: PhantomData<Data>,
}
// needed to work around lifetimes :/
pub struct Decrypted<Data> {
    data: Vec<u8>,
    data_type: PhantomData<Data>,
}

impl<'de, Data: Deserialize<'de>> Decrypted<Data> {
    pub fn deserialize(&'de self) -> Data {
        bincode::deserialize(&self.data).unwrap()
    }
}

impl<'de, Data: Serialize + Deserialize<'de>> Encrypted<Data> {
    pub fn decrypt(&self, key: Key<Aes256Gcm>) -> Result<Decrypted<Data>, DecryptionError> {
        let cipher = Aes256Gcm::new(&key);
        let decrypt = cipher
            .decrypt(
                GenericArray::from_slice(self.nonce.as_slice()),
                self.data.as_ref(),
            )
            .map_err(|_| DecryptionError::Decryption)?;
        Ok(Decrypted {
            data: decrypt,
            data_type: PhantomData,
        })
    }

    pub fn encrypt(data: &Data, key: Key<Aes256Gcm>) -> anyhow::Result<Encrypted<Data>> {
        let cipher = Aes256Gcm::new(&key);
        let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
        let encoded = bincode::serialize(data)?;
        let encrypted = cipher
            .encrypt(&nonce, encoded.as_ref())
            .map_err(|_| EncryptionError::Encryption)?;
        Ok(Encrypted {
            data: encrypted,
            nonce: nonce.to_vec(),
            data_type: PhantomData,
        })
    }
}

pub trait SecureData {
    type Item;
    fn salt(&self) -> &str;
    fn data(&self) -> &Encrypted<Self::Item>;
    // not much point in this function
    fn encrypt<'de>(data: &Self::Item, key: Key<Aes256Gcm>) -> anyhow::Result<Encrypted<Self::Item>>
    where
        Self::Item: Serialize + Deserialize<'de>,
    {
        Encrypted::encrypt(data, key)
    }
    fn decrypt<'de>(&self, key: Key<Aes256Gcm>) -> anyhow::Result<Decrypted<Self::Item>>
    where
        Self::Item: Serialize + Deserialize<'de> + 'de,
    {
        let res = Encrypted::decrypt(self.data(), key)?;
        Ok(res)
    }
    // not much point in this fuction
    //
    // fn deserialize<'de>(decrypted: &'de Decrypted<Self::Item>) -> Self::Item
    // where
    //     Self::Item: Deserialize<'de>,
    // {
    //     Decrypted::deserialize(decrypted)
    // }
    fn key(&self, password: String) -> Key<Aes256Gcm> {
        Self::get_key(self.salt(), password)
    }
    fn get_key(salt: &str, password: String) -> Key<Aes256Gcm> {
        let salt_string = SaltString::from_b64(salt).unwrap();
        let mut salt_arr = [0u8; 64];
        let salt_bytes = salt_string.decode_b64(&mut salt_arr).unwrap();

        let mut output_key = [0u8; 32];
        let argon2 = Argon2::default();
        argon2
            .hash_password_into(password.as_bytes(), salt_bytes, &mut output_key)
            .unwrap();

        output_key.into()
    }
}
