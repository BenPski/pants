use std::{
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

pub trait ProjectFile<'de, Data>
where
    Data: Serialize + Deserialize<'de>,
{
    fn base_path(&self) -> PathBuf;

    fn path(&self) -> PathBuf;
    fn create(&self) -> anyhow::Result<File> {
        let path = self.path();
        if let Some(dir) = path.parent() {
            fs::create_dir_all(dir)?;
        }

        let file = File::create(path)?;
        Ok(file)
    }

    fn delete(&self) -> anyhow::Result<()> {
        let path = self.path();
        Ok(fs::remove_file(path)?)
    }

    fn open(&self) -> anyhow::Result<File> {
        let path = self.path();
        let file = File::open(path)?;
        Ok(file)
    }
    // NOTE: Couldn't figure out making the reading and writing generic with serde
    // also making all the trait inheritance work with blanket implementations was
    // too much of a headache, all of which just seemed better to copy and paste the
    // implementations
    fn write(&mut self, data: &Data) -> anyhow::Result<()> {
        let mut file = self.create()?;
        let output = serde_json::to_string(data)?;
        file.write_all(output.as_ref())
            .map_err(|_| SaveError::Write)?;

        Ok(())
    }

    fn read(&self) -> anyhow::Result<ReadIn<Data>> {
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
    base_path: PathBuf,
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
    base_path: PathBuf,
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
        let mut path = self.base_path();
        path.push(self.name.clone());
        path.push(format!("{}-{}", self.name, format_date(self.timestamp)));
        path.set_extension("json");
        path
    }

    fn base_path(&self) -> PathBuf {
        self.base_path.to_path_buf()
    }
}

impl<'de, Data> ProjectFile<'de, Data> for NonTimestampedFile<Data>
where
    Data: Serialize + Deserialize<'de>,
{
    fn path(&self) -> PathBuf {
        let mut path = self.base_path();
        path.push(self.name.clone());
        path.push(self.name.clone());
        path.set_extension("json");
        path
    }

    fn base_path(&self) -> PathBuf {
        self.base_path.to_path_buf()
    }
}

impl<'a, Data> TimestampedFile<Data>
where
    Self: Name,
    Data: Serialize + Deserialize<'a>,
{
    fn new(base_path: PathBuf, timestamp: DateTime<Local>) -> Self {
        Self {
            name: Self::name(),
            base_path,
            timestamp,
            data_type: PhantomData,
        }
    }

    fn now(base_path: PathBuf) -> Self {
        Self::new(base_path, now())
    }
}

impl<'a, Data> NonTimestampedFile<Data>
where
    Self: Name,
    Data: Serialize + Deserialize<'a>,
{
    fn new(base_path: PathBuf) -> Self {
        Self {
            name: Self::name(),
            base_path,
            data_type: PhantomData,
        }
    }

    pub fn exists(&self) -> bool {
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

pub struct SaveDir {
    base_path: PathBuf,
}

impl SaveDir {
    pub fn new(base_path: PathBuf) -> Self {
        Self { base_path }
    }

    pub fn remove(&self) -> Result<(), std::io::Error> {
        fs::remove_dir_all(self.base_path.to_path_buf())
    }

    pub fn vault_file(&self) -> VaultFile {
        self.nontimestamped_file()
    }

    pub fn schema_file(&self) -> SchemaFile {
        self.nontimestamped_file()
    }

    pub fn record_file(&self) -> RecordFile {
        self.timestamped_file()
    }

    pub fn record_file_latest(&self) -> Option<RecordFile> {
        self.timestamped_file_recent()
    }

    pub fn record_file_all(&self) -> Vec<RecordFile> {
        self.timestamped_file_all()
    }

    pub fn backup_file(&self) -> BackupFile {
        self.timestamped_file()
    }

    pub fn backup_file_latest(&self) -> Option<BackupFile> {
        self.timestamped_file_recent()
    }

    pub fn backup_file_all(&self) -> Vec<BackupFile> {
        self.timestamped_file_all()
    }

    fn nontimestamped_file<'de, Data>(&self) -> NonTimestampedFile<Data>
    where
        NonTimestampedFile<Data>: Name,
        Data: Serialize + Deserialize<'de>,
    {
        NonTimestampedFile::new(self.base_path.to_path_buf())
    }

    fn timestamped_file<'de, Data>(&self) -> TimestampedFile<Data>
    where
        TimestampedFile<Data>: Name,
        Data: Serialize + Deserialize<'de>,
    {
        TimestampedFile::now(self.base_path.to_path_buf())
    }

    fn timestamped_file_all<'de, Data>(&self) -> Vec<TimestampedFile<Data>>
    where
        TimestampedFile<Data>: Name,
        Data: Serialize + Deserialize<'de>,
    {
        let mut path = self.base_path.clone();
        path.push(&TimestampedFile::name());
        path.push(format!("{}-*.json", TimestampedFile::name()));
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
                Ok(t) => paths.push(TimestampedFile::new(self.base_path.to_path_buf(), t)),
            }
        }
        paths
    }

    fn timestamped_file_recent<'de, Data>(&self) -> Option<TimestampedFile<Data>>
    where
        TimestampedFile<Data>: Name,
        Data: Serialize + Deserialize<'de>,
    {
        let mut path = self.base_path.clone();
        path.push(&TimestampedFile::name());
        path.push(format!("{}-*.json", TimestampedFile::name()));
        glob(path.to_str().unwrap())
            .expect("Failed to read glob pattern")
            .fold(None, |acc, entry| match entry {
                Ok(p) => {
                    let file_name = p.file_stem().unwrap().to_str().unwrap();
                    let split = file_name.split_once('-').unwrap();
                    let time = read_date(split.1).unwrap();
                    match acc {
                        None => Some(TimestampedFile::new(self.base_path.to_path_buf(), time)),
                        Some(ref f) => {
                            if f.timestamp < time {
                                Some(TimestampedFile::new(self.base_path.to_path_buf(), time))
                            } else {
                                acc
                            }
                        }
                    }
                }
                _ => acc,
            })
    }
}
