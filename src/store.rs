use std::{collections::HashMap, fmt::Display, hash::Hash};

use enum_iterator::{all, Sequence};
use inquire::Confirm;
use pants_gen::password::Password;
use serde::{Deserialize, Serialize};

use crate::errors::SchemaError;

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
    pub fn convert(&self, data: &HashMap<String, String>) -> Option<Store> {
        match self {
            Self::Password => {
                let p = data.get("password")?;
                Some(Store::Password(p.to_string()))
            }
            Self::UsernamePassword => {
                let p = data.get("password")?;
                let u = data.get("username")?;
                Some(Store::UsernamePassword(u.to_string(), p.to_string()))
            }
        }
    }

    pub fn convert_default(&self) -> Store {
        match self {
            Self::Password => Store::Password(String::new()),
            Self::UsernamePassword => Store::UsernamePassword(String::new(), String::new()),
        }
    }

    pub fn all() -> Vec<StoreChoice> {
        all::<StoreChoice>().collect()
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
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

    pub fn split(&self) -> (StoreChoice, HashMap<String, String>) {
        match self {
            Self::Password(p) => {
                let mut map = HashMap::new();
                map.insert("password".to_string(), p.to_string());
                (StoreChoice::Password, map)
            }
            Self::UsernamePassword(u, p) => {
                let mut map = HashMap::new();
                map.insert("password".to_string(), p.to_string());
                map.insert("username".to_string(), u.to_string());
                (StoreChoice::UsernamePassword, map)
            }
        }
    }

    pub fn choice(&self) -> StoreChoice {
        self.split().0
    }

    pub fn as_hash(&self) -> HashMap<String, String> {
        self.split().1
    }

    // // from schema type and field values
    // pub fn from_fields(
    //     repr: &str,
    //     value: HashMap<String, String>,
    // ) -> Result<Store, Box<dyn Error>> {
    //     match repr {
    //         "password" => {
    //             if let Some(p) = value.get("password") {
    //                 Ok(Self::Password(p.to_string()))
    //             } else {
    //                 Err(Box::new(SchemaError::BadValues))
    //             }
    //         }
    //         _ => Err(Box::new(SchemaError::BadType)),
    //     }
    // }
    //
    // // from  schema type and array of values
    // pub fn from_array(repr: &str, value: Vec<String>) -> Result<Store, Box<dyn Error>> {
    //     match repr {
    //         "password" => {
    //             if let Some(p) = value.first() {
    //                 Ok(Self::Password(p.to_string()))
    //             } else {
    //                 Err(Box::new(SchemaError::BadValues))
    //             }
    //         }
    //         _ => Err(Box::new(SchemaError::BadType)),
    //     }
    // }

    pub fn prompt(repr: &str) -> anyhow::Result<Store> {
        match repr {
            "password" => Self::get_password().map(Store::Password),
            "username-password" => {
                let username_input = inquire::Text::new("Username:")
                    .with_help_message("New username")
                    .prompt();
                let username = username_input?;
                let password = Self::get_password()?;
                Ok(Store::UsernamePassword(username, password))
            }
            _ => Err(Box::new(SchemaError::BadType).into()),
        }
    }

    fn get_password() -> anyhow::Result<String> {
        let generate = Confirm::new("Generate password?")
            .with_default(true)
            .with_help_message("Create a random password or enter manually?")
            .prompt();
        match generate {
            Ok(true) => {
                let length_input = inquire::CustomType::<usize>::new("Length of password?")
                    .with_help_message("Number of characters use in the generated password")
                    .with_error_message("Please type a valid number")
                    .with_default(32)
                    .prompt()?;
                let upper = inquire::Confirm::new("Use uppercase letters?")
                    .with_default(true)
                    .with_help_message(
                        "Use the uppercase alphabetic characters (A-Z) in password generation",
                    )
                    .prompt()?;
                let lower = inquire::Confirm::new("Use lowercase letters?")
                    .with_default(true)
                    .with_help_message(
                        "Use the lowercase alphabetic characters (a-z) in password generation",
                    )
                    .prompt()?;
                let numbers = inquire::Confirm::new("Use numbers?")
                    .with_default(true)
                    .with_help_message("Use the numbers (0-9) in password generation")
                    .prompt()?;
                let symbols = inquire::Confirm::new("Use symbols?")
                    .with_default(true)
                    .with_help_message("Use special symbols in password generation")
                    .prompt()?;

                let mut spec = Password::new().length(length_input);

                if upper {
                    spec = spec.upper_at_least(1);
                }
                if lower {
                    spec = spec.lower_at_least(1);
                }
                if numbers {
                    spec = spec.number_at_least(1);
                }
                if symbols {
                    spec = spec.symbol_at_least(1);
                }
                let password = spec.generate().unwrap();
                Ok(password)
            }
            Ok(false) => {
                let password_input = inquire::Password::new("Password: ")
                    .with_display_toggle_enabled()
                    .with_display_mode(inquire::PasswordDisplayMode::Masked)
                    .prompt();
                match password_input {
                    Ok(p) => Ok(p),
                    Err(_) => Err(Box::new(SchemaError::BadValues).into()),
                }
            }
            Err(_) => Err(Box::new(SchemaError::BadValues).into()),
        }
    }
}
