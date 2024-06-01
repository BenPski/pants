use std::{cell::RefCell, error::Error, rc::Rc};

use aes_gcm::{Aes256Gcm, Key};
use argon2::password_hash::SaltString;
use rand::rngs::OsRng;

use crate::{
    command::{Command, Commands},
    file::{BackupFile, ProjectFile, RecordFile, SchemaFile, VaultFile},
    message::Message,
    output::Output,
    reads::Reads,
    schema::Schema,
    secure::{Encrypted, SecureData},
    store::Store,
    Password,
};

use super::{
    encrypted::{RecordEncrypted, VaultEncrypted},
    Vault,
};

pub struct VaultInterface {
    vault: Vault,
    vault_encrypted: VaultEncrypted,
    key: Key<Aes256Gcm>,
    // schema: Schema,
    record: RecordEncrypted,
    schema_file: Rc<RefCell<SchemaFile>>,
    vault_file: Rc<RefCell<VaultFile>>,
    record_file: Rc<RefCell<RecordFile>>,
}

impl VaultInterface {
    pub fn receive(message: Message) -> Result<Output, Box<dyn Error>> {
        match message {
            Message::Get(password, key) => {
                let command = Command::Read { key };
                let mut interface = Self::load_interface(password)?;
                let reads = interface.transaction(command.into())?;
                Ok(reads.into())
            }
            Message::Update(password, key, value) => {
                let command = Command::Update { key, value };
                let mut interface = Self::load_interface(password)?;
                let reads = interface.transaction(command.into())?;
                Ok(reads.into())
            }
            Message::Delete(password, key) => {
                let command = Command::Delete { key };
                let mut interface = Self::load_interface(password)?;
                let _reads = interface.transaction(command.into())?;
                Ok(().into())
            }
            Message::Backup(password) => {
                let interface = Self::load_interface(password)?;
                let backup = interface.backup()?;
                Ok(Output::Backup(backup))
            }
            Message::Rotate(password, new_password) => {
                let mut interface = Self::load_interface(password)?;
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

                let mut interface = Self::load_interface(password)?;

                // have proved that the user knows the backup's and current vault's password and
                // the decryption of both, so make a backup of the current vault and then copy in
                // the old vault as the current vault
                let new_backup = interface.backup()?;

                interface.vault_encrypted = backup_vault_enc;
                interface.key = backup_key;
                interface.save()?;
                Ok(Output::Backup(new_backup))
            }
            Message::Schema => {
                let schema = VaultInterface::get_schema();
                Ok(Output::Schema(schema))
            }
            Message::BackupList => {
                let files = BackupFile::all();
                Ok(Output::BackupFiles(files))
            }
        }
    }

    fn get_schema() -> Schema {
        if let Ok(data) = SchemaFile::default().read() {
            data.deserialize()
        } else {
            Schema::default()
        }
    }

    fn load_interface(password: Password) -> Result<VaultInterface, Box<dyn Error>> {
        let mut interface = Self::get_interface(password)?;
        interface.check_unfinished()?;
        Ok(interface)
    }

    fn get_interface(password: Password) -> Result<Self, Box<dyn Error>> {
        let vault_file = VaultFile::default();
        let record_file = RecordFile::default();
        let schema_file = SchemaFile::default();
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
            vault_file: Rc::new(RefCell::new(vault_file)),
            record_file: Rc::new(RefCell::new(record_file)),
            schema_file: Rc::new(RefCell::new(schema_file)),
        })
    }

    fn check_unfinished(&mut self) -> Result<(), Box<dyn Error>> {
        if let Some(file) = RecordFile::last() {
            self.apply_unfinished(file)?
        }

        Ok(())
    }

    fn apply_unfinished(&mut self, record_file: RecordFile) -> Result<(), Box<dyn Error>> {
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

    fn save(&mut self) -> Result<(), Box<dyn Error>> {
        self.vault_encrypted.update(&self.vault, self.key)?;
        self.vault_file.borrow_mut().write(&self.vault_encrypted)?;
        self.schema_file.borrow_mut().write(&self.vault.schema())?;
        Ok(())
    }

    fn backup(&self) -> Result<BackupFile, Box<dyn Error>> {
        let mut backup_file = BackupFile::default();
        let backup = VaultEncrypted {
            salt: self.vault_encrypted.salt.clone(),
            data: Encrypted::encrypt(&self.vault, self.key)?,
        };
        backup_file.write(&backup)?;
        Ok(backup_file)
    }

    fn transaction(&mut self, commands: Commands) -> Result<Reads<Store>, Box<dyn Error>> {
        let (reads, record) = self.vault.transaction(commands);
        self.record.update(&record, self.key)?;

        self.record_file.borrow_mut().write(&self.record)?;
        self.vault.apply_record(record);
        self.save()?;
        self.record_file.borrow_mut().delete()?;
        Ok(reads)
    }
}

// pub struct VaultInterface {
//     vault_file: VaultFile,
//     schema_file: Rc<RefCell<SchemaFile>>,
//     vault: VaultEncrypted,
//     record: RecordEncrypted,
//     key: Key<Aes256Gcm>,
// }
//
// impl VaultInterface {
//     pub fn create(
//         vault_file: VaultFile,
//         schema_file: Rc<RefCell<SchemaFile>>,
//         password: String,
//     ) -> VaultInterface {
//         let vault = VaultEncrypted::new(password.clone()).unwrap();
//         let record = RecordEncrypted::new(password.clone()).unwrap();
//         let key = vault.key(password);
//         Self {
//             vault_file,
//             schema_file,
//             vault,
//             record,
//             key,
//         }
//     }
//
//     pub fn open(
//         vault_file: VaultFile,
//         schema_file: Rc<RefCell<SchemaFile>>,
//         password: String,
//     ) -> Result<VaultInterface, Box<dyn Error>> {
//         let vault: VaultEncrypted = vault_file.read()?.deserialize();
//         let record = RecordEncrypted::new(password.clone()).unwrap();
//         let key = vault.key(password);
//         Ok(Self {
//             vault_file,
//             schema_file,
//             vault,
//             record,
//             key,
//         })
//     }
//
//     // TODO: working around this function is clunky and error prone, refactor
//     fn save(&mut self) -> Result<(), Box<dyn Error>> {
//         self.vault_file.write(&self.vault)?;
//         self.schema_file
//             .borrow_mut()
//             .write(&self.vault.decrypt(self.key)?.deserialize().schema())?;
//         Ok(())
//     }
//
//     fn transact(
//         transaction: Commands,
//         schema_file: Rc<RefCell<SchemaFile>>,
//     ) -> Result<Output, Box<dyn Error>> {
//         let mut interface = Self::get_interface(schema_file)?;
//         let mut vault = interface.vault.decrypt(interface.key)?.deserialize();
//         let (reads, record) = vault.transaction(transaction);
//         interface.record.update(&record, interface.key)?;
//
//         let mut record_file = RecordFile::default();
//         record_file.write(&interface.record)?;
//         vault.apply_record(record);
//         interface.vault.update(&vault, interface.key)?;
//         interface.save()?;
//         record_file.delete()?;
//         Ok(reads.into())
//     }
//
//     fn interact(
//         interaction: Interaction,
//         schema_file: Rc<RefCell<SchemaFile>>,
//     ) -> Result<Output, Box<dyn Error>> {
//         match interaction {
//             Interaction::List => {
//                 let schema = schema_file.borrow_mut().read()?.deserialize();
//                 Ok(Output::List(schema.all_info()))
//             }
//             Interaction::Backup => {
//                 let interface = Self::get_interface(schema_file)?;
//                 let backup = interface.backup()?;
//
//                 Ok(Output::Backup(backup.path()))
//             }
//             Interaction::BackupList => {
//                 let backups = BackupFile::all();
//                 Ok(Output::List(
//                     backups.into_iter().map(|b| b.to_string()).collect(),
//                 ))
//             }
//             Interaction::BackupRestore => {
//                 let backups = BackupFile::all();
//                 let backup_file =
//                     inquire::Select::new("Which backup to restore?", backups).prompt()?;
//
//                 let backup_password = inquire::Password::new("Backup's password:")
//                     .with_display_toggle_enabled()
//                     .with_display_mode(inquire::PasswordDisplayMode::Masked)
//                     .without_confirmation()
//                     .prompt()?;
//                 let backup_vault_enc = backup_file.read()?.deserialize();
//                 let backup_key = backup_vault_enc.key(backup_password);
//                 let _backup_vault = backup_vault_enc.decrypt(backup_key)?.deserialize();
//
//                 let mut interface = Self::get_interface(schema_file)?;
//                 let _vault = interface.vault.decrypt(interface.key)?.deserialize();
//
//                 // have proved that the user knows the backup's and current vault's password and
//                 // the decryption of both, so make a backup of the current vault and then copy in
//                 // the old vault as the current vault
//                 let new_backup = interface.backup()?;
//
//                 interface.vault = backup_vault_enc;
//                 interface.key = backup_key;
//                 interface.save()?;
//                 Ok(Output::Backup(new_backup.path()))
//             }
//             Interaction::Rotate => {
//                 // TODO: does extra unnecessary decryption
//                 let mut interface = Self::get_interface(schema_file)?;
//                 let backup = interface.backup()?;
//                 let vault = interface.vault.decrypt(interface.key)?.deserialize();
//                 let new_password = inquire::Password::new("New vault password: ")
//                     .with_display_toggle_enabled()
//                     .with_display_mode(inquire::PasswordDisplayMode::Masked)
//                     .prompt()?;
//                 let mut new_vault = VaultEncrypted::new(new_password.clone())?;
//                 new_vault.update(&vault, new_vault.key(new_password.clone()))?;
//                 interface.vault = new_vault;
//                 interface.key = interface.vault.key(new_password);
//                 interface.save()?;
//                 Ok(Output::Backup(backup.path()))
//             }
//         }
//     }
//
//     fn backup(&self) -> Result<BackupFile, Box<dyn Error>> {
//         let vault = self.vault.decrypt(self.key)?.deserialize();
//         let mut backup_file = BackupFile::default();
//         let backup = VaultEncrypted {
//             salt: self.vault.salt.clone(),
//             data: Encrypted::encrypt(&vault, self.key)?,
//         };
//         backup_file.write(&backup)?;
//         Ok(backup_file)
//     }
//
//     fn check_unfinished(&mut self) -> Result<(), Box<dyn Error>> {
//         if let Some(file) = RecordFile::last() {
//             let ans = inquire::Confirm::new("There appears to be an unapplied update, apply it? Will clear out old record if not applied.")
//                 .with_default(true)
//                 .with_help_message("Likely occurred due to some failure in saving off the updates from the previous interaction.")
//                 .prompt();
//
//             match ans {
//                 Ok(true) => self.apply_unfinished(file)?,
//                 Ok(false) => file.delete()?,
//                 Err(_) => (),
//             }
//         }
//
//         Ok(())
//     }
//
//     fn apply_unfinished(&mut self, record_file: RecordFile) -> Result<(), Box<dyn Error>> {
//         let mut vault = self.vault.decrypt(self.key)?.deserialize();
//         let record = record_file
//             .read()?
//             .deserialize()
//             .decrypt(self.key)?
//             .deserialize();
//         vault.apply_record(record);
//         self.vault.update(&vault, self.key)?;
//         self.save()?;
//         record_file.delete()?;
//         Ok(())
//     }
//
//     fn get_interface(
//         schema_file: Rc<RefCell<SchemaFile>>,
//     ) -> Result<VaultInterface, Box<dyn Error>> {
//         let vault_file = VaultFile::default();
//         let mut interface = if vault_file.check() {
//             let password = inquire::Password::new("Vault password: ")
//                 .without_confirmation()
//                 .with_display_toggle_enabled()
//                 .with_display_mode(inquire::PasswordDisplayMode::Masked)
//                 .prompt()?;
//             let res = Self::open(vault_file, schema_file.clone(), password)?;
//
//             Ok::<VaultInterface, Box<dyn Error>>(res)
//         } else {
//             let new_password = inquire::Password::new("Vault doesn't exist, create password: ")
//                 .with_display_toggle_enabled()
//                 .with_display_mode(inquire::PasswordDisplayMode::Masked)
//                 .prompt()?;
//             let mut interface = Self::create(vault_file, schema_file.clone(), new_password);
//             interface.save()?;
//             Ok(interface)
//         }?;
//
//         interface.check_unfinished()?;
//         Ok(interface)
//     }
//
//     pub fn interaction(transaction: CLICommands) -> Result<Output, Box<dyn Error>> {
//         let schema_file: Rc<RefCell<SchemaFile>> = Rc::new(RefCell::new(SchemaFile::default()));
//         let schema = if schema_file.borrow().check() {
//             schema_file.borrow_mut().read()?.deserialize()
//         } else {
//             // ensure a file exists since it can get used later
//             // TODO: should rework to make this not necessary
//             let s = Schema::default();
//             schema_file.borrow_mut().write(&s)?;
//             s
//         };
//
//         let commands = Instructions::from_commands(transaction, schema)?;
//         match commands {
//             Instructions::Interaction(i) => Self::interact(i, schema_file),
//             Instructions::Commands(c) => Self::transact(c, schema_file),
//         }
//     }
// }
