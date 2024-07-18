use iced::{
    widget::{button, column, container, text, text_input},
    Element, Length,
};
use secrecy::ExposeSecret;

use pants_store::Password;

use crate::{gui_message::GUIMessage, widget::helpers::form, INPUT_ID};

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
    pub fn confirm() -> Self {
        Self::new(true)
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
        let password_input = text_input("vault password", self.password.clone().expose_secret())
            .id(INPUT_ID.clone())
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
        form(header, container(column![password_input, cancel]))
            .max_width(500.0)
            .into()
    }
}
