use iced::{
    widget::{button, column, container, text, text_input},
    Element, Length,
};
use iced_aw::Card;

use crate::gui::gui_message::GUIMessage;

#[derive(Debug, Clone, Default)]
pub struct PasswordState {
    pub password: String,
}

impl PasswordState {
    pub fn view(&self) -> Element<GUIMessage> {
        let header = text("Vault password");
        let password_input = text_input("vault password", &self.password.clone())
            .on_input(GUIMessage::PasswordChanged)
            .on_submit(GUIMessage::Submit)
            .width(Length::Fill)
            .secure(true);
        let cancel = button("Cancel").on_press(GUIMessage::Exit);
        Card::new(header, container(column![password_input, cancel]))
            .max_width(500.0)
            .into()
    }
}
