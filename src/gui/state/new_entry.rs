
use iced::{
    widget::{button, column, container, pick_list, row, text, text_input},
    Element, Length,
};
use iced_aw::Card;
use secrecy::ExposeSecret;

use crate::{
    gui::gui_message::GUIMessage,
    store::{StoreChoice, StoreHash},
};

#[derive(Debug, Clone)]
pub struct NewEntryState {
    pub vault: String,
    pub name: String,
    pub choice: StoreChoice,
    pub value: StoreHash,
}

impl Default for NewEntryState {
    fn default() -> Self {
        NewEntryState {
            vault: String::new(),
            name: String::new(),
            choice: StoreChoice::default(),
            value: StoreChoice::default().convert_default().as_hash(),
        }
    }
}

impl NewEntryState {
    pub fn for_vault(vault: String) -> Self {
        NewEntryState {
            vault,
            name: String::new(),
            choice: StoreChoice::default(),
            value: StoreChoice::default().convert_default().as_hash(),
        }
    }
    pub fn view(&self) -> Element<GUIMessage> {
        let header = text(format!("New entry for {}", self.vault));
        let name_prefix = text("Name:");
        let name_input = text_input("Name", &self.name).on_input(GUIMessage::ChangeName);
        let style_choice = pick_list(
            StoreChoice::all(),
            Some(self.choice),
            GUIMessage::SelectStyle,
        );
        let data_input = match &self.choice {
            StoreChoice::Password => {
                let prefix = text("Password:");
                let password_input = text_input(
                    "Password",
                    self.value.get("password").unwrap().expose_secret(),
                )
                .width(Length::Fill)
                .on_input(|v| GUIMessage::UpdateField("password".to_string(), v.into()));
                let password_generate = button("Generate").on_press(GUIMessage::GeneratePassword);

                container(row![prefix, password_input, password_generate])
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
                .on_input(|v| GUIMessage::UpdateField("password".to_string(), v.into()));

                let password_generate = button("Generate").on_press(GUIMessage::GeneratePassword);
                container(column![
                    row![username_prefix, username_input],
                    row![password_prefix, password_input, password_generate]
                ])
            }
        };
        let create_button = button("Create").on_press(GUIMessage::Submit);
        let cancel_button = button("Cancel").on_press(GUIMessage::Exit);
        Card::new(
            header,
            container(column![
                row![name_prefix, name_input],
                style_choice,
                data_input,
                row![create_button, cancel_button]
            ]),
        )
        .max_width(500.0)
        .into()
    }
}
