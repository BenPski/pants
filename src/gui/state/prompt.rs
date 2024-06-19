use iced::{
    widget::{button, column, container, row, text, text_input},
    Element,
};
use iced_aw::Card;

use crate::gui::gui_message::GUIMessage;

#[derive(Debug, Clone)]
pub struct PromptState {
    pub vault: String,
}

impl Default for PromptState {
    fn default() -> Self {
        PromptState {
            vault: String::new(),
        }
    }
}

impl PromptState {
    pub fn view(&self) -> Element<GUIMessage> {
        let header = text("New vault name");
        let name_input = text_input("Name", &self.vault).on_input(GUIMessage::PromptChanged);

        let create_button = button("Create").on_press(GUIMessage::Submit);
        let cancel_button = button("Cancel").on_press(GUIMessage::Exit);
        Card::new(
            header,
            container(column![name_input, row![create_button, cancel_button]]),
        )
        .max_width(500.0)
        .into()
    }
}
