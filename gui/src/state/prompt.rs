use iced::{
    widget::{button, column, container, row, text, text_input},
    Element,
};

use crate::{gui_message::GUIMessage, widget::card::Card, INPUT_ID};

#[derive(Debug, Clone, Default)]
pub struct PromptState {
    pub vault: String,
}

impl PromptState {
    pub fn view(&self) -> Element<GUIMessage> {
        let header = text("New vault name");
        let name_input = text_input("Name", &self.vault)
            .id(INPUT_ID.clone())
            .on_input(GUIMessage::PromptChanged)
            .on_submit(GUIMessage::Submit);

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
