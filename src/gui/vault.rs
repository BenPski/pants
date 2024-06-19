use iced::{
    alignment,
    widget::{button, column, container, row, text},
    Element, Length,
};

use super::entry::{Entry, EntryMessage};

pub struct Vault {
    pub name: String,
    pub entries: Vec<Entry>,
}

#[derive(Debug, Clone)]
pub enum VaultMessage {
    Entry(EntryMessage, String),
    NewEntry,
    Delete,
}

impl Vault {
    pub fn new(name: String, entries: Vec<Entry>) -> Self {
        Self { name, entries }
    }

    pub fn view(&self) -> Element<VaultMessage> {
        let name = text(self.name.to_string()).size(20).width(Length::Fill);
        let delete_button = button("X").on_press(VaultMessage::Delete);
        let header = row![name, delete_button];
        let content = container(column(self.entries.iter().map(|e| {
            e.view()
                .map(move |message| VaultMessage::Entry(message, e.key.clone()))
        })))
        .padding(10);
        let new_button = button(
            text("+")
                .horizontal_alignment(alignment::Horizontal::Center)
                .width(Length::Fill),
        )
        .width(Length::Fill)
        .on_press(VaultMessage::NewEntry);
        container(column![header, content, new_button])
            .padding(10)
            .into()
    }
}
