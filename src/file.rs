use std::{
    error::Error,
    fmt::Display,
    fs::{self, File},
    io::{Read, Write},
    marker::PhantomData,
    path::PathBuf,
};

use chrono::{DateTime, Local};
use glob::glob;
use serde::{Deserialize, Serialize};

use crate::{
    errors::SaveError,
    schema::Schema,
    utils::{format_date, now, read_date},
    vault::encrypted::{RecordEncrypted, VaultEncrypted},
};

// TODO: create encrypted files that need to be given a password/key to open
// applies to the vault, vault backup, and record files

pub trait ProjectFile<'de, Data>
where
    Data: Serialize + Deserialize<'de>,
{
    fn base_path() -> PathBuf {
        if let Some(project_dirs) = directories_next::ProjectDirs::from("com", "bski", "pants") {
            project_dirs.data_dir().into()
        } else {
            std::env::current_dir().unwrap_or_default()
        }
    }

    fn path(&self) -> PathBuf;
    fn create(&self) -> Result<File, Box<dyn Error>> {
        let path = self.path();
        if let Some(dir) = path.parent() {
            fs::create_dir_all(dir)?;
        }

        let file = File::create(path)?;
        Ok(file)
    }

    fn delete(&self) -> Result<(), Box<dyn Error>> {
        let path = self.path();
        Ok(fs::remove_file(path)?)
    }

    fn open(&self) -> Result<File, Box<dyn Error>> {
        let path = self.path();
        let file = File::open(path)?;
        Ok(file)
    }
    // NOTE: Couldn't figure out making the reading and writing generic with serde
    // also making all the trait inheritance work with blanket implementations was
    // too much of a headache, all of which just seemed better to copy and paste the
    // implementations
    fn write(&mut self, data: &Data) -> Result<(), Box<dyn Error>> {
        let mut file = self.create()?;
        let output = serde_json::to_string(data)?;
        file.write_all(output.as_ref())
            .map_err(|_| SaveError::Write)?;

        Ok(())
    }

    fn read(&self) -> Result<ReadIn<Data>, Box<dyn Error>> {
        let mut file = self.open()?;
        let mut content = String::new();
        file.read_to_string(&mut content)?;
        Ok(ReadIn {
            data: content,
            data_type: PhantomData,
        })
    }
}

pub struct ReadIn<Data> {
    data: String,
    data_type: PhantomData<Data>,
}

impl<'de, Data: Deserialize<'de>> ReadIn<Data> {
    pub fn deserialize(&'de self) -> Data {
        serde_json::from_str(&self.data).unwrap()
    }
}

#[derive(Debug, Clone)]
pub struct TimestampedFile<Data> {
    name: String,
    timestamp: DateTime<Local>,
    data_type: PhantomData<Data>,
}

impl<'de, Data> Display for TimestampedFile<Data>
where
    Data: Serialize + Deserialize<'de>,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.path())
    }
}

#[derive(Debug, Clone)]
pub struct NonTimestampedFile<Data> {
    name: String,
    data_type: PhantomData<Data>,
}

impl<'de, Data> Display for NonTimestampedFile<Data>
where
    Data: Serialize + Deserialize<'de>,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.path())
    }
}

impl<'de, Data> ProjectFile<'de, Data> for TimestampedFile<Data>
where
    Data: Serialize + Deserialize<'de>,
{
    fn path(&self) -> PathBuf {
        let mut path = Self::base_path();
        path.push(self.name.clone());
        path.push(format!("{}-{}", self.name, format_date(self.timestamp)));
        path.set_extension("json");
        path
    }
}

impl<'de, Data> ProjectFile<'de, Data> for NonTimestampedFile<Data>
where
    Data: Serialize + Deserialize<'de>,
{
    fn path(&self) -> PathBuf {
        let mut path = Self::base_path();
        path.push(self.name.clone());
        path.push(self.name.clone());
        path.set_extension("json");
        path
    }
}

impl<'a, Data> TimestampedFile<Data>
where
    Self: Name,
    Data: Serialize + Deserialize<'a>,
{
    fn new(timestamp: DateTime<Local>) -> Self {
        Self {
            name: Self::name(),
            timestamp,
            data_type: PhantomData,
        }
    }

    fn now() -> Self {
        Self::new(now())
    }

    pub fn last() -> Option<Self> {
        let mut path = Self::base_path();
        path.push(&Self::name());
        path.push(format!("{}-*.json", Self::name()));
        glob(path.to_str().unwrap())
            .expect("Failed to read glob pattern")
            .fold(None, |acc, entry| match entry {
                Ok(p) => {
                    let file_name = p.file_stem().unwrap().to_str().unwrap();
                    let split = file_name.split_once('-').unwrap();
                    let time = read_date(split.1).unwrap();
                    match acc {
                        None => Some(Self::new(time)),
                        Some(ref f) => {
                            if f.timestamp < time {
                                Some(Self::new(time))
                            } else {
                                acc
                            }
                        }
                    }
                }
                _ => acc,
            })
    }

    pub fn all() -> Vec<Self> {
        let mut path = Self::base_path();
        path.push(&Self::name());
        path.push(format!("{}-*.json", Self::name()));
        let mut paths = vec![];
        for entry in glob(path.to_str().unwrap())
            .expect("Failed to read glob pattern")
            .flatten()
        {
            let file_name = entry.file_stem().unwrap().to_str().unwrap();
            let split = file_name.split_once('-').unwrap();
            let _name = split.0.to_owned();
            let timestamp = read_date(split.1);
            match timestamp {
                Err(err) => println!("Malformed timestamp in filename: {:?}. {:?}", entry, err),
                Ok(t) => paths.push(Self::new(t)),
            }
        }
        paths
    }
}

impl<'a, Data> NonTimestampedFile<Data>
where
    Self: Name,
    Data: Serialize + Deserialize<'a>,
{
    fn new() -> Self {
        Self {
            name: Self::name(),
            data_type: PhantomData,
        }
    }

    pub fn check(&self) -> bool {
        self.path().exists()
    }
}

pub type VaultFile = NonTimestampedFile<VaultEncrypted>;
pub type RecordFile = TimestampedFile<RecordEncrypted>;
pub type BackupFile = TimestampedFile<VaultEncrypted>;
pub type SchemaFile = NonTimestampedFile<Schema>;

pub trait Name {
    fn name() -> String;
}

impl Name for VaultFile {
    fn name() -> String {
        "vault".to_string()
    }
}

impl Name for RecordFile {
    fn name() -> String {
        "record".to_string()
    }
}

impl Name for BackupFile {
    fn name() -> String {
        "backup".to_string()
    }
}

impl Name for SchemaFile {
    fn name() -> String {
        "schema".to_string()
    }
}

impl<'a, Data> Default for TimestampedFile<Data>
where
    TimestampedFile<Data>: Name,
    Data: Serialize + Deserialize<'a>,
{
    fn default() -> Self {
        Self::now()
    }
}

impl<'a, Data> Default for NonTimestampedFile<Data>
where
    NonTimestampedFile<Data>: Name,
    Data: Serialize + Deserialize<'a>,
{
    fn default() -> Self {
        Self::new()
    }
}
