use iced::{
    widget::{button, column, container, row, text, text_input},
    Element, Length,
};
use secrecy::ExposeSecret;

use crate::{
    gui::{gui_message::GUIMessage, widget::card::Card, INPUT_ID},
    store::{Store, StoreChoice, StoreHash},
    Password,
};

#[derive(Debug, Clone)]
pub struct EntryState {
    pub vault: String,
    pub key: String,
    pub choice: StoreChoice,
    pub value: StoreHash,
    pub hidden: bool,
}

impl EntryState {
    pub fn view(&self) -> Element<GUIMessage> {
        let header = text(format!("{} in {}", self.key.clone(), self.vault));
        let data_input = match &self.choice {
            StoreChoice::Password => {
                let prefix = text("Password:");
                let password_input = text_input(
                    "Password",
                    self.value.get("password").unwrap().expose_secret(),
                )
                .id(INPUT_ID.clone())
                .width(Length::Fill)
                .on_input(|v| GUIMessage::UpdateField("password".to_string(), v.into()))
                .secure(self.hidden);
                let show_button = if self.hidden {
                    button("Show").on_press(GUIMessage::ShowPassword)
                } else {
                    button("Hide").on_press(GUIMessage::HidePassword)
                };
                let copy_button = button("Copy").on_press(GUIMessage::CopyPassword);
                let password_generate = button("Generate").on_press(GUIMessage::GeneratePassword);
                container(row![
                    prefix,
                    password_input,
                    password_generate,
                    copy_button,
                    show_button
                ])
            }
            StoreChoice::UsernamePassword => {
                let username_prefix = text("Username:");
                let password_prefix = text("Password:");
                let username_input = text_input(
                    "Username",
                    self.value.get("username").unwrap().expose_secret(),
                )
                .width(Length::Fill)
                .on_input(|v| GUIMessage::UpdateField("username".to_string(), v.into()));
                let password_input = text_input(
                    "Password",
                    self.value.get("password").unwrap().expose_secret(),
                )
                .width(Length::Fill)
                .on_input(|v| GUIMessage::UpdateField("password".to_string(), v.into()))
                .secure(self.hidden);
                let show_button = if self.hidden {
                    button("Show").on_press(GUIMessage::ShowPassword)
                } else {
                    button("Hide").on_press(GUIMessage::HidePassword)
                };
                let copy_button = button("Copy").on_press(GUIMessage::CopyPassword);
                let password_generate = button("Generate").on_press(GUIMessage::GeneratePassword);
                container(column![
                    row![username_prefix, username_input],
                    row![
                        password_prefix,
                        password_input,
                        password_generate,
                        copy_button,
                        show_button
                    ]
                ])
            }
        };

        let save_button = button("Save").on_press(GUIMessage::Submit);
        let done_button = button("Done").on_press(GUIMessage::Exit);
        Card::new(
            header,
            container(column![data_input, row![save_button, done_button]]),
        )
        .max_width(500.0)
        .into()
    }

    pub fn update(&mut self, value: Store) {
        let (choice, value) = value.split();
        self.choice = choice;
        self.value = value;
    }

    pub fn get_password(&self) -> Option<Password> {
        for (key, value) in self.value.iter() {
            if key == "password" {
                return Some(value.clone());
            }
        }
        None
    }

    pub fn from_entry(vault: String, key: String, style: String) -> Self {
        let value = match style.as_str() {
            "password" => Store::Password(String::new().into()),
            "username-password" => {
                Store::UsernamePassword(String::new().into(), String::new().into())
            }
            _ => panic!("unrecognized entry value {}", style),
        };
        let (choice, value) = value.split();
        EntryState {
            vault,
            key,
            choice,
            value,
            hidden: true,
        }
    }
}
