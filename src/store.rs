use std::{collections::HashMap, fmt::Display};

use enum_iterator::{all, Sequence};
use secrecy::{ExposeSecret, Secret, SerializableSecret};
use serde::{Deserialize, Serialize};
use zeroize::ZeroizeOnDrop;

use crate::Password;

pub type StoreHash = HashMap<String, String>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Sequence)]
pub enum StoreChoice {
    Password,
    UsernamePassword,
}

impl Display for StoreChoice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StoreChoice::Password => write!(f, "Password"),
            StoreChoice::UsernamePassword => write!(f, "Username/Password"),
        }
    }
}

impl Default for StoreChoice {
    fn default() -> Self {
        Self::UsernamePassword
    }
}

impl StoreChoice {
    pub fn convert(&self, data: &StoreHash) -> Option<Store> {
        match self {
            Self::Password => {
                let p = data.get("password")?;
                Some(Store::Password(p.clone()))
            }
            Self::UsernamePassword => {
                let p = data.get("password")?;
                let u = data.get("username")?;
                Some(Store::UsernamePassword(u.clone(), p.clone()))
            }
        }
    }

    pub fn convert_default(&self) -> Store {
        match self {
            Self::Password => Store::Password(String::new()),
            Self::UsernamePassword => Store::UsernamePassword(
                String::new(),
                String::new(),
                // StoreValue::Secret(String::new().into()),
            ),
        }
    }

    pub fn all() -> Vec<StoreChoice> {
        all::<StoreChoice>().collect()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ZeroizeOnDrop)]
pub enum Store {
    Password(String),
    UsernamePassword(String, String),
}

impl Display for Store {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Password(p) => write!(f, "{}", p),
            Self::UsernamePassword(username, password) => write!(f, "{}: {}", username, password),
        }
    }
}

impl Store {
    // how to represent the type in the schema
    pub fn repr(&self) -> String {
        match self {
            Self::Password(_) => "password".to_string(),
            Self::UsernamePassword(_, _) => "username-password".to_string(),
        }
    }

    pub fn split(&self) -> (StoreChoice, StoreHash) {
        match self {
            Self::Password(p) => {
                let mut map = HashMap::new();
                map.insert("password".to_string(), p.clone());
                (StoreChoice::Password, map)
            }
            Self::UsernamePassword(u, p) => {
                let mut map = HashMap::new();
                map.insert("password".to_string(), p.clone());
                map.insert("username".to_string(), u.clone());
                (StoreChoice::UsernamePassword, map)
            }
        }
    }

    pub fn choice(&self) -> StoreChoice {
        self.split().0
    }

    pub fn as_hash(&self) -> StoreHash {
        self.split().1
    }

    // pub fn expose(&self) -> StoreOpen {
    //     match self {
    //         Self::Password(StoreValue::Secret(p)) => StoreOpen::Password(p.expose_secret().into()),
    //         Self::UsernamePassword(StoreValue::Value(u), StoreValue::Secret(p)) => {
    //             StoreOpen::UsernamePassword(u.into(), p.expose_secret().into())
    //         }
    //         _ => panic!("Malformed store"),
    //     }
    // }
}
