use crate::{file::BackupFile, info::Info, reads::Reads, schema::Schema, store::Store};

#[derive(Debug, Clone)]
pub enum Output {
    Info(Info),
    Schema(Schema),
    BackupFiles(Vec<BackupFile>),
    Read(Reads<Store>),
    List(Vec<String>),
    Backup(BackupFile),
    Nothing,
}

impl From<Vec<BackupFile>> for Output {
    fn from(value: Vec<BackupFile>) -> Self {
        Output::BackupFiles(value)
    }
}

impl From<Schema> for Output {
    fn from(value: Schema) -> Self {
        Output::Schema(value)
    }
}

impl From<Reads<Store>> for Output {
    fn from(value: Reads<Store>) -> Self {
        Output::Read(value)
    }
}

impl From<Vec<String>> for Output {
    fn from(value: Vec<String>) -> Self {
        Output::List(value)
    }
}

impl From<()> for Output {
    fn from(_value: ()) -> Self {
        Output::Nothing
    }
}

impl From<Info> for Output {
    fn from(value: Info) -> Self {
        Output::Info(value)
    }
}
//
// impl Output {
//     fn finish(&self) -> anyhow::Result<()> {
//         match self {
//             Self::Read(reads) => {
//                 if !reads.data.is_empty() {
//                     let mut clipboard = Clipboard::new()?;
//                     let orig = clipboard.get_text().unwrap_or("".to_string());
//                     for (key, value) in reads.data.clone().into_iter() {
//                         println!("{}", key);
//                         match value {
//                             Store::Password(pass) => {
//                                 clipboard.set_text(pass)?;
//                                 println!("  password: <Copied to clipboard>");
//                                 thread::sleep(Duration::from_secs(5));
//                             }
//                             Store::UsernamePassword(user, pass) => {
//                                 clipboard.set_text(pass)?;
//                                 println!("  username: {}", user);
//                                 println!("  password: <Copied to clipboard>");
//                                 thread::sleep(Duration::from_secs(5));
//                             }
//                         }
//                     }
//                     clipboard.set_text(orig)?;
//                     println!("Resetting clipboard");
//                     thread::sleep(Duration::from_secs(1));
//                 } else {
//                     println!("Nothing read from vault");
//                 }
//
//                 Ok(())
//             }
//             Self::List(items) => {
//                 if items.is_empty() {
//                     println!("No entries");
//                 } else {
//                     println!("Available entries:");
//                     for item in items {
//                         println!("- {}", item);
//                     }
//                 }
//                 Ok(())
//             }
//             Self::Backup(path) => {
//                 println!("Backed up to {}", path);
//                 Ok(())
//             }
//             Self::Nothing => Ok(()),
//             Output::Schema(schema) => {
//                 println!("{}", schema);
//                 Ok(())
//             }
//             Output::BackupFiles(files) => {
//                 for file in files {
//                     println!("{}", file);
//                 }
//                 Ok(())
//             }
//         }
//     }
// }

// impl Display for Output {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         match self {
//             Self::Info(info) => {
//                 for (vault, schema) in info.data.iter() {
//                     writeln!(f, "{}:", vault)?;
//                     for (key, value) in schema.data.iter() {
//                         writeln!(f, "  {}: {}", key, value)?;
//                     }
//                 }
//                 Ok(())
//             }
//             Self::Nothing => write!(f, ""),
//             Self::Backup(path) => write!(f, "Backed up to {}", path),
//             Self::List(items) => {
//                 if items.is_empty() {
//                     write!(f, "No entries")
//                 } else {
//                     for item in items {
//                         writeln!(f, "{}", item)?;
//                     }
//                     Ok(())
//                 }
//             }
//             Self::Read(reads) => {
//                 write!(f, "{}", reads)
//             }
//             Self::BackupFiles(items) => {
//                 for item in items {
//                     writeln!(f, "{}", item)?;
//                 }
//                 Ok(())
//             }
//             Self::Schema(schema) => {
//                 for (key, value) in schema.data.iter() {
//                     writeln!(f, "{}: {}", key, value)?;
//                 }
//                 Ok(())
//             }
//         }
//     }
// }
