use std::fmt::Display;

use boring_derive::From;
use enum_iterator::Sequence;
use secrecy::{CloneableSecret, DebugSecret, Secret, SerializableSecret, Zeroize};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Sequence, Serialize, Deserialize)]
pub enum StoreType {
    Password,
    UsernamePassword,
    Generic,
}

#[derive(Clone, Serialize, Deserialize, From)]
pub struct StoredValue(String);

impl StoredValue {
    pub fn new(s: impl Into<String>) -> Self {
        StoredValue(s.into())
    }
}

impl Display for StoredValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Zeroize for StoredValue {
    fn zeroize(&mut self) {
        self.0.zeroize();
    }
}

impl CloneableSecret for StoredValue {}
impl DebugSecret for StoredValue {}
// need to be able to serialize for the general encryption
impl SerializableSecret for StoredValue {}

pub type SecretValue = Secret<StoredValue>;

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct Store {
    // pub ty: StoreType,
    pub data: Vec<(String, SecretValue)>,
}

impl Store {
    // some convenience things
    pub fn password(pass: impl Into<SecretValue>) -> Self {
        Self {
            // ty: StoreType::Password,
            data: vec![("Password".into(), pass.into())],
        }
    }
    pub fn username_password(
        username: impl Into<SecretValue>,
        pass: impl Into<SecretValue>,
    ) -> Self {
        Self {
            // ty: StoreType::UsernamePassword,
            data: vec![
                ("Username".into(), username.into()),
                ("Password".into(), pass.into()),
            ],
        }
    }
    pub fn new(data: impl Into<Vec<(String, SecretValue)>>) -> Self {
        Self {
            // ty,
            data: data.into(),
        }
    }

    pub fn insert(&mut self, key: &str, value: SecretValue) {
        if let Some(v) = self.get_mut(key) {
            *v = value
        } else {
            self.data.push((key.into(), value));
        }
    }

    pub fn get(&self, key: &str) -> Option<&SecretValue> {
        for (k, v) in &self.data {
            if k == key {
                return Some(v);
            }
        }
        None
    }

    pub fn get_mut(&mut self, key: &str) -> Option<&mut SecretValue> {
        for (k, v) in self.data.iter_mut() {
            if k == key {
                return Some(v);
            }
        }
        None
    }

    pub fn remove(&mut self, key: &str) -> Option<(String, SecretValue)> {
        let pos = self.data.iter().position(|(k, _)| k == key)?;
        Some(self.data.remove(pos))
    }

    pub fn join(&mut self, extension: Store) {
        for (k, v) in extension.data {
            self.insert(&k, v);
        }
    }

    pub fn update(&self, changes: Changes) -> Self {
        let mut new = Self::default();
        for (k, v) in changes.data {
            let value = v.or_else(|| self.get(&k).cloned());
            if let Some(value) = value {
                new.insert(&k, value);
            }
        }
        new
    }
}

/// represents the changes during an update to the [Store]
#[derive(Debug, Clone)]
pub struct Changes {
    data: Vec<(String, Option<SecretValue>)>,
}

impl Changes {
    pub fn new(fields: &[String]) -> Self {
        let data = fields.iter().map(|s| (s.into(), None)).collect();
        Self { data }
    }

    pub fn fields(&self) -> Vec<String> {
        self.data.iter().map(|(k, _)| k.to_string()).collect()
    }

    pub fn insert(&mut self, key: &str, value: SecretValue) {
        if let Some(v) = self.get_mut(key) {
            *v = Some(value)
        } else {
            self.data.push((key.into(), Some(value)));
        }
    }

    // fn get(&self, key: &str) -> Option<&Option<SecretValue>> {
    //     for (k, v) in &self.data {
    //         if k == key {
    //             return Some(v);
    //         }
    //     }
    //     None
    // }

    fn get_mut(&mut self, key: &str) -> Option<&mut Option<SecretValue>> {
        for (k, v) in self.data.iter_mut() {
            if k == key {
                return Some(v);
            }
        }
        None
    }
    pub fn remove(&mut self, key: &str) -> Option<(String, Option<SecretValue>)> {
        let pos = self.data.iter().position(|(k, _)| k == key)?;
        Some(self.data.remove(pos))
    }
    pub fn swap(&mut self, first: &str, second: &str) {
        let mut pos1 = 0;
        let mut pos2 = 0;
        for (i, (k, _)) in self.data.iter().enumerate() {
            if k == first {
                pos1 = i;
            }
            if k == second {
                pos2 = i;
            }
        }
        self.data.swap(pos1, pos2);
    }
    pub fn move_to(&mut self, first: &str, second: &str) {
        let mut pos1 = 0;
        let mut pos2 = 0;
        for (i, (k, _)) in self.data.iter().enumerate() {
            if k == first {
                pos1 = i;
            }
            if k == second {
                pos2 = i;
            }
        }
        let temp = self.data[pos1].clone();
        self.data.remove(pos1);
        self.data.insert(pos2, temp);
    }

    pub fn unchanged(&self, fields: &[String]) -> bool {
        let mut curr = Vec::new();
        for (k, v) in &self.data {
            if v.is_some() {
                return true;
            }
            curr.push(k.to_string());
        }
        curr == fields
    }
}
