use iced::{
    widget::{button, column, container, pick_list, row, text, text_input},
    Element, Length,
};
use secrecy::ExposeSecret;

use crate::{
    gui::{gui_message::GUIMessage, widget::card::Card, INPUT_ID},
    store::{StoreChoice, StoreHash},
};

#[derive(Debug, Clone)]
pub struct NewEntryState {
    pub vault: String,
    pub name: String,
    pub choice: StoreChoice,
    pub value: StoreHash,
    pub hidden: bool,
}

impl Default for NewEntryState {
    fn default() -> Self {
        NewEntryState {
            vault: String::new(),
            name: String::new(),
            choice: StoreChoice::default(),
            value: StoreChoice::default().convert_default().as_hash(),
            hidden: true,
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
            hidden: true,
        }
    }
    pub fn view(&self) -> Element<GUIMessage> {
        let header = text(format!("New entry for {}", self.vault));
        let name_prefix = text("Name:");
        let name_input = text_input("Name", &self.name)
            .on_input(GUIMessage::ChangeName)
            .on_submit(GUIMessage::Submit)
            .id(INPUT_ID.clone());
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
                .on_input(|v| GUIMessage::UpdateField("password".to_string(), v.into()))
                .on_submit(GUIMessage::Submit)
                .secure(self.hidden);
                let password_generate = button("Generate").on_press(GUIMessage::GeneratePassword);
                let toggle_show = if self.hidden {
                    button("Show").on_press(GUIMessage::ShowPassword)
                } else {
                    button("Hide").on_press(GUIMessage::HidePassword)
                };

                container(row![prefix, password_input, password_generate, toggle_show])
            }
            StoreChoice::UsernamePassword => {
                let username_prefix = text("Username:");
                let password_prefix = text("Password:");
                let username_input = text_input(
                    "Username",
                    self.value.get("username").unwrap().expose_secret(),
                )
                .width(Length::Fill)
                .on_input(|v| GUIMessage::UpdateField("username".to_string(), v.into()))
                .on_submit(GUIMessage::Submit);
                let password_input = text_input(
                    "Password",
                    self.value.get("password").unwrap().expose_secret(),
                )
                .width(Length::Fill)
                .on_input(|v| GUIMessage::UpdateField("password".to_string(), v.into()))
                .on_submit(GUIMessage::Submit)
                .secure(self.hidden);

                let password_generate = button("Generate").on_press(GUIMessage::GeneratePassword);
                let toggle_show = if self.hidden {
                    button("Show").on_press(GUIMessage::ShowPassword)
                } else {
                    button("Hide").on_press(GUIMessage::HidePassword)
                };
                container(column![
                    row![username_prefix, username_input],
                    row![
                        password_prefix,
                        password_input,
                        password_generate,
                        toggle_show
                    ]
                ])
            }
        };
        let create_button = button("Create").on_press(GUIMessage::Submit);
        let cancel_button = button("Cancel").on_press(GUIMessage::Exit);
        // let header = container(header).style(|theme: &iced::Theme| {
        //     let palette = theme.extended_palette();
        //     container::Appearance::default().with_background(palette.background.weak.color)
        // });
        // let card = column![
        //     header,
        //     container(column![
        //         row![name_prefix, name_input],
        //         style_choice,
        //         data_input,
        //         row![create_button, cancel_button]
        //     ])
        // ];
        // let card = container(card)
        //     .style(|theme: &iced::Theme| {
        //         let palette = theme.extended_palette();
        //         container::Appearance::default().with_border(palette.primary.strong.color, 1.0)
        //     })
        //     .max_width(500.0);
        // card.into()
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
