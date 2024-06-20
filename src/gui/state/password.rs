use iced::{
    widget::{button, column, container, text, text_input},
    Element, Length,
};
use iced_aw::Card;
use secrecy::ExposeSecret;

use crate::{gui::gui_message::GUIMessage, Password};

#[derive(Debug, Clone)]
pub struct PasswordState {
    pub password: Password,
    pub confirm: Option<Password>,
}

impl Default for PasswordState {
    fn default() -> Self {
        Self {
            password: String::new().into(),
            confirm: None,
        }
    }
}

impl PasswordState {
    pub fn new(confirm: bool) -> Self {
        PasswordState {
            password: String::new().into(),
            confirm: if confirm {
                Some(String::new().into())
            } else {
                None
            },
        }
    }
    pub fn valid(&self) -> bool {
        if let Some(confirm) = &self.confirm {
            self.password.expose_secret() == confirm.expose_secret()
        } else {
            true
        }
    }
    pub fn view(&self) -> Element<GUIMessage> {
        let header = text("Vault password");
        let password_input = text_input("vault password", &self.password.clone().expose_secret())
            .on_input(|p| GUIMessage::PasswordChanged(p.into()))
            .on_submit(GUIMessage::Submit)
            .width(Length::Fill)
            .secure(true);
        let password_input = if let Some(confirm) = &self.confirm {
            let confirm_input = text_input("confirm password", confirm.expose_secret())
                .on_input(|p| GUIMessage::PasswordConfirmChanged(p.into()))
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
