use std::{error::Error, fmt::Display};

use inquire::Confirm;
use pants_gen::password::Password;
use serde::{Deserialize, Serialize};

use crate::errors::SchemaError;

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

    pub fn prompt(repr: &str) -> Result<Store, Box<dyn Error>> {
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
            _ => Err(Box::new(SchemaError::BadType)),
        }
    }

    fn get_password() -> Result<String, Box<dyn Error>> {
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
                    Err(_) => Err(Box::new(SchemaError::BadValues)),
                }
            }
            Err(_) => Err(Box::new(SchemaError::BadValues)),
        }
    }
}
