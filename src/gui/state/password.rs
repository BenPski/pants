use iced::{
    widget::{button, column, container, text, text_input},
    Element, Length,
};
use iced_aw::Card;

use crate::gui::gui_message::GUIMessage;

#[derive(Debug, Clone, Default)]
pub struct PasswordState {
    pub password: String,
    pub confirm: Option<String>,
}

impl PasswordState {
    pub fn new(confirm: bool) -> Self {
        PasswordState {
            password: String::new(),
            confirm: if confirm { Some(String::new()) } else { None },
        }
    }
    pub fn valid(&self) -> bool {
        if let Some(confirm) = &self.confirm {
            self.password == *confirm
        } else {
            true
        }
    }
    pub fn view(&self) -> Element<GUIMessage> {
        let header = text("Vault password");
        let password_input = text_input("vault password", &self.password.clone())
            .on_input(GUIMessage::PasswordChanged)
            .on_submit(GUIMessage::Submit)
            .width(Length::Fill)
            .secure(true);
        let password_input = if let Some(confirm) = &self.confirm {
            let confirm_input = text_input("confirm password", confirm)
                .on_input(GUIMessage::PasswordConfirmChanged)
                .on_submit(GUIMessage::Submit)
                .width(Length::Fill)
                .secure(true);
            column![password_input, confirm_input]
        } else {
            column![password_input]
        };
        let cancel = button("Cancel").on_press(GUIMessage::Exit);
        Card::new(header, container(column![password_input, cancel]))
            .max_width(500.0)
            .into()
    }
}
