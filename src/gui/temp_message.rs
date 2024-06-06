use std::collections::HashMap;

use iced::{
    widget::{container, text},
    Element,
};

use crate::{message::Message, store::StoreChoice};

use super::gui_message::GUIMessage;

#[derive(Debug, Clone, Default)]
pub enum TempMessage {
    #[default]
    Empty,
    Delete(String),
    Get(String),
    New(String, StoreChoice, HashMap<String, String>),
    Update(String, StoreChoice, HashMap<String, String>),
}

impl TempMessage {
    pub fn needs_password(&self) -> bool {
        match self {
            Self::Empty => false,
            Self::Delete(_) => true,
            Self::Get(_) => true,
            Self::New(_, _, _) => true,
            Self::Update(_, _, _) => true,
        }
    }

    pub fn complete(&self) -> bool {
        match self {
            Self::Empty => true,
            Self::New(name, _, fields) => {
                let mut filled = true;
                for (_, value) in fields.iter() {
                    if value.is_empty() {
                        filled = false;
                        break;
                    }
                }
                !name.is_empty() && filled
            }
            Self::Update(name, _, fields) => {
                let mut filled = true;
                for (_, value) in fields.iter() {
                    if value.is_empty() {
                        filled = false;
                        break;
                    }
                }
                !name.is_empty() && filled
            }
            Self::Get(name) => !name.is_empty(),
            Self::Delete(name) => !name.is_empty(),
        }
    }

    pub fn with_password(&self, password: String) -> Message {
        match self {
            Self::Delete(key) => Message::Delete(password, key.to_string()),
            Self::Get(key) => Message::Get(password, key.to_string()),
            Self::New(key, choice, value) => {
                Message::Update(password, key.clone(), choice.convert(value).unwrap())
            }
            Self::Update(key, choice, value) => {
                Message::Update(password, key.clone(), choice.convert(value).unwrap())
            }
            Self::Empty => Message::Schema,
        }
    }

    pub fn view(&self) -> Element<GUIMessage> {
        match self {
            TempMessage::Delete(key) => {
                let info = text(format!("Working on deleting {}", key));
                container(info).into()
            }
            TempMessage::Get(key) => {
                let info = text(format!("Working on getting {}", key));
                container(info).into()
            }
            TempMessage::New(key, _, _) => {
                let info = text(format!("Working on a new entry {}", key));
                container(info).into()
            }
            TempMessage::Update(key, _, _) => {
                let info = text(format!("Working on updating entry {}", key));
                container(info).into()
            }
            Self::Empty => {
                let info = text("Working on nothing");
                container(info).into()
            }
        }
    }
}
