use std::error::Error;

use aes_gcm::{aead::OsRng, Aes256Gcm, Key};
use argon2::password_hash::SaltString;
use serde::{Deserialize, Serialize};

use crate::{
    action::Record,
    secure::{Encrypted, SecureData},
    vault::Vault,
};

#[derive(Debug, Serialize, Deserialize)]
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
    pub fn new(password: String) -> Result<Self, Box<dyn Error>> {
        let salt = SaltString::generate(&mut OsRng).to_string();
        let key = Self::get_key(&salt, password);
        Encrypted::encrypt(&Vault::new(), key).map(|vault| Self { data: vault, salt })
    }

    pub fn from_vault(
        salt: String,
        key: Key<Aes256Gcm>,
        vault: &Vault,
    ) -> Result<Self, Box<dyn Error>> {
        Encrypted::encrypt(vault, key).map(|vault| Self { data: vault, salt })
    }

    pub fn update(&mut self, data: &Vault, key: Key<Aes256Gcm>) -> Result<(), Box<dyn Error>> {
        let updated = Encrypted::encrypt(data, key)?;
        self.data = updated;
        Ok(())
    }
}

pub type RecordEncrypted = PasswordEncrypted<Record>;

impl RecordEncrypted {
    pub fn new(password: String) -> Result<Self, Box<dyn Error>> {
        let salt = SaltString::generate(&mut OsRng).to_string();
        let key = Self::get_key(&salt, password);
        Encrypted::encrypt(&Record::new(), key).map(|vault| Self { data: vault, salt })
    }

    pub fn update(&mut self, data: &Record, key: Key<Aes256Gcm>) -> Result<(), Box<dyn Error>> {
        let updated = Encrypted::encrypt(data, key)?;
        self.data = updated;
        Ok(())
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
//             let ans = Confirm::new("There appears to be an unapplied update, apply it? Will clear out old record if not applied.")
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
//             let s = Schema::new(HashMap::new());
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
