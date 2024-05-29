use std::{error::Error, fmt::Display, path::PathBuf, thread, time::Duration};

use arboard::Clipboard;

use crate::{reads::Reads, store::Store};

#[derive(Debug)]
pub enum Output {
    Read(Reads<Store>),
    List(Vec<String>),
    Backup(PathBuf),
    Nothing,
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

impl Output {
    pub fn finish(&self) -> Result<(), Box<dyn Error>> {
        match self {
            Self::Read(reads) => {
                if !reads.data.is_empty() {
                    let mut clipboard = Clipboard::new()?;
                    let orig = clipboard.get_text().unwrap_or("".to_string());
                    for (key, value) in reads.data.clone().into_iter() {
                        println!("{}", key);
                        match value {
                            Store::Password(pass) => {
                                clipboard.set_text(pass)?;
                                println!("  password: <Copied to clipboard>");
                                thread::sleep(Duration::from_secs(5));
                            }
                            Store::UsernamePassword(user, pass) => {
                                clipboard.set_text(pass)?;
                                println!("  username: {}", user);
                                println!("  password: <Copied to clipboard>");
                                thread::sleep(Duration::from_secs(5));
                            }
                        }
                    }
                    clipboard.set_text(orig)?;
                    println!("Resetting clipboard");
                    thread::sleep(Duration::from_secs(1));
                } else {
                    println!("Nothing read from vault");
                }

                Ok(())
            }
            Self::List(items) => {
                if items.is_empty() {
                    println!("No entries");
                } else {
                    println!("Available entries:");
                    for item in items {
                        println!("- {}", item);
                    }
                }
                Ok(())
            }
            Self::Backup(path) => {
                println!("Backed up to {}", path.to_str().unwrap());
                Ok(())
            }
            _ => Ok(()),
        }
    }
}

impl Display for Output {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Nothing => write!(f, ""),
            Self::Backup(path) => write!(f, "Backed up to {}", path.to_str().unwrap()),
            Self::List(items) => {
                if items.is_empty() {
                    write!(f, "No entries")
                } else {
                    for item in items {
                        writeln!(f, "{}", item)?;
                    }
                    Ok(())
                }
            }
            Self::Read(reads) => {
                write!(f, "{}", reads)
            }
        }
    }
}
