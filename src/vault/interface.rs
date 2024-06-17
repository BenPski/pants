use core::panic;
use std::{cell::RefCell, path::PathBuf, rc::Rc};

use aes_gcm::{Aes256Gcm, Key};
use argon2::password_hash::SaltString;
use rand::rngs::OsRng;

use crate::{
    command::{Command, Commands},
    config::{internal_config::InternalConfig, vault_config::VaultConfig},
    file::{BackupFile, ProjectFile, RecordFile, SaveDir, SchemaFile, VaultFile},
    message::Message,
    output::Output,
    reads::Reads,
    schema::Schema,
    secure::{Encrypted, SecureData},
    store::Store,
    utils, Password,
};

use super::{
    encrypted::{RecordEncrypted, VaultEncrypted},
    Vault,
};

pub struct VaultInterface {
    config: VaultConfig,
}
//
// impl Default for VaultInterface {
//     fn default() -> Self {
//         Self::new(utils::base_path())
//     }
// }

impl VaultInterface {
    pub fn new(save_dir: PathBuf) -> Self {
        let config = VaultConfig::new(save_dir);

        Self { config }
    }
    pub fn receive(&self, message: Message) -> anyhow::Result<Output> {
        match message {
            Message::Schema => Ok(self.get_schema().into()),
            Message::BackupList => Ok(self.config.save_dir().backup_file_all().into()),
            _ => VaultHandler::receive(message, self.config.save_dir()),
        }
    }

    fn get_schema(&self) -> Schema {
        let schema_file: SchemaFile = self.config.save_dir().schema_file();
        schema_file
            .read()
            .map(|data| data.deserialize())
            .unwrap_or(Schema::default())
    }
}

pub struct VaultHandler {
    vault: Vault,
    vault_encrypted: VaultEncrypted,
    key: Key<Aes256Gcm>,
    // schema: Schema,
    record: RecordEncrypted,
    save_dir: SaveDir,
    schema_file: Rc<RefCell<SchemaFile>>,
    vault_file: Rc<RefCell<VaultFile>>,
    record_file: Rc<RefCell<RecordFile>>,
}

impl VaultHandler {
    pub fn receive(message: Message, save_dir: SaveDir) -> anyhow::Result<Output> {
        match message {
            Message::Get(password, key) => {
                let command = Command::Read { key };
                let mut interface = Self::load_interface(password, save_dir)?;
                let reads = interface.transaction(command.into())?;
                Ok(reads.into())
            }
            Message::Update(password, key, value) => {
                let command = Command::Update { key, value };
                let mut interface = Self::load_interface(password, save_dir)?;
                let reads = interface.transaction(command.into())?;
                Ok(reads.into())
            }
            Message::Delete(password, key) => {
                let command = Command::Delete { key };
                let mut interface = Self::load_interface(password, save_dir)?;
                let _reads = interface.transaction(command.into())?;
                Ok(().into())
            }
            Message::Backup(password) => {
                let interface = Self::load_interface(password, save_dir)?;
                let backup = interface.backup()?;
                Ok(Output::Backup(backup))
            }
            Message::Rotate(password, new_password) => {
                let mut interface = Self::load_interface(password, save_dir)?;
                let backup = interface.backup()?;
                let new_vault = VaultEncrypted::new(new_password.clone())?;
                let key = new_vault.key(new_password);
                interface.vault_encrypted = new_vault;
                interface.key = key;
                interface.save()?;
                Ok(Output::Backup(backup))
            }
            Message::Restore(password, backup_password, backup_file) => {
                let backup_vault_enc = backup_file.read()?.deserialize();
                let backup_key = backup_vault_enc.key(backup_password);
                let _backup_vault = backup_vault_enc.decrypt(backup_key)?.deserialize();

                let mut interface = Self::load_interface(password, save_dir)?;

                // have proved that the user knows the backup's and current vault's password and
                // the decryption of both, so make a backup of the current vault and then copy in
                // the old vault as the current vault
                let new_backup = interface.backup()?;

                interface.vault_encrypted = backup_vault_enc;
                interface.key = backup_key;
                interface.save()?;
                Ok(Output::Backup(new_backup))
            }
            _ => panic!("Should have been caught by handler"),
        }
    }

    fn load_interface(password: Password, save_dir: SaveDir) -> anyhow::Result<Self> {
        let mut interface = Self::get_interface(password, save_dir)?;
        interface.check_unfinished()?;
        Ok(interface)
    }

    fn get_interface(password: Password, save_dir: SaveDir) -> anyhow::Result<Self> {
        let vault_file = save_dir.vault_file();
        let record_file = save_dir.record_file();
        let schema_file = save_dir.schema_file();
        // let schema = Self::get_schema();
        let record = RecordEncrypted::new(password.clone())?;
        let (vault, key, vault_encrypted) = if vault_file.exists() {
            let vault_encrypted = vault_file.read()?.deserialize();
            let key = vault_encrypted.key(password);
            let vault = vault_encrypted.decrypt(key)?.deserialize();
            (vault, key, vault_encrypted)
        } else {
            let vault = Vault::new();
            let salt = SaltString::generate(&mut OsRng);
            let key = VaultEncrypted::get_key(salt.as_str(), password);
            let vault_encrypted = VaultEncrypted::from_vault(salt.to_string(), key, &vault)?;
            (vault, key, vault_encrypted)
        };

        Ok(Self {
            vault,
            vault_encrypted,
            key,
            // schema,
            record,
            save_dir,
            vault_file: Rc::new(RefCell::new(vault_file)),
            record_file: Rc::new(RefCell::new(record_file)),
            schema_file: Rc::new(RefCell::new(schema_file)),
        })
    }

    fn check_unfinished(&mut self) -> anyhow::Result<()> {
        if let Some(file) = self.save_dir.record_file_latest() {
            self.apply_unfinished(file)?
        }

        Ok(())
    }

    fn apply_unfinished(&mut self, record_file: RecordFile) -> anyhow::Result<()> {
        let record = record_file
            .read()?
            .deserialize()
            .decrypt(self.key)?
            .deserialize();
        self.vault.apply_record(record);
        self.save()?;
        record_file.delete()?;
        Ok(())
    }

    fn save(&mut self) -> anyhow::Result<()> {
        self.vault_encrypted.update(&self.vault, self.key)?;
        self.vault_file.borrow_mut().write(&self.vault_encrypted)?;
        self.schema_file.borrow_mut().write(&self.vault.schema())?;
        Ok(())
    }

    fn backup(&self) -> anyhow::Result<BackupFile> {
        let mut backup_file = self.save_dir.backup_file();
        let backup = VaultEncrypted {
            salt: self.vault_encrypted.salt.clone(),
            data: Encrypted::encrypt(&self.vault, self.key)?,
        };
        backup_file.write(&backup)?;
        Ok(backup_file)
    }

    fn transaction(&mut self, commands: Commands) -> anyhow::Result<Reads<Store>> {
        let (reads, record) = self.vault.transaction(commands);
        self.record.update(&record, self.key)?;

        self.record_file.borrow_mut().write(&self.record)?;
        self.vault.apply_record(record);
        self.save()?;
        self.record_file.borrow_mut().delete()?;
        Ok(reads)
    }
}
