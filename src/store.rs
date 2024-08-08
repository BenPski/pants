use std::{collections::HashMap, fmt::Display};

use enum_iterator::{all, Sequence};
use secrecy::{ExposeSecret, Secret};
use serde::{
    ser::{SerializeMap, SerializeTupleVariant},
    Deserialize, Serialize,
};

pub type StoreHash = HashMap<String, Secret<String>>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Sequence)]
pub enum StoreChoice {
    Password,
    UsernamePassword,
    Generic,
}

impl Display for StoreChoice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StoreChoice::Password => write!(f, "Password"),
            StoreChoice::UsernamePassword => write!(f, "Username/Password"),
            StoreChoice::Generic => write!(f, "Generic"),
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
            Self::Generic => Some(Store::Generic(data.clone())),
        }
    }

    pub fn convert_default(&self) -> Store {
        match self {
            Self::Password => Store::Password(String::new().into()),
            Self::UsernamePassword => Store::UsernamePassword(
                String::new().into(),
                String::new().into(),
                // StoreValue::Secret(String::new().into()),
            ),
            Self::Generic => Store::Generic(HashMap::new()),
        }
    }

    pub fn all() -> Vec<StoreChoice> {
        all::<StoreChoice>().collect()
    }
}

#[derive(Debug, Clone, Deserialize)]
pub enum Store {
    Password(Secret<String>),
    UsernamePassword(Secret<String>, Secret<String>),
    Generic(HashMap<String, Secret<String>>),
}

impl Serialize for Store {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Self::Password(p) => {
                let mut state = serializer.serialize_tuple_variant("Store", 0, "Password", 1)?;
                state.serialize_field(p.expose_secret())?;
                state.end()
            }
            Self::UsernamePassword(u, p) => {
                let mut state =
                    serializer.serialize_tuple_variant("Store", 1, "UsernamePassword", 2)?;
                state.serialize_field(u.expose_secret())?;
                state.serialize_field(p.expose_secret())?;
                state.end()
            }
            Self::Generic(m) => {
                let mut state = serializer.serialize_map(Some(m.len()))?;
                for (k, v) in m {
                    state.serialize_entry(k, v.expose_secret())?;
                }
                state.end()
            }
        }
    }
}

// impl Display for Store {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         match self {
//             Self::Password(p) => write!(f, "{}", p),
//             Self::UsernamePassword(username, password) => write!(f, "{}: {}", username, password),
//         }
//     }
// }

impl Store {
    // how to represent the type in the schema
    pub fn repr(&self) -> String {
        match self {
            Self::Password(_) => "password".to_string(),
            Self::UsernamePassword(_, _) => "username-password".to_string(),
            Self::Generic(_) => "generic".to_string(),
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
            Self::Generic(m) => (StoreChoice::Generic, m.clone()),
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
