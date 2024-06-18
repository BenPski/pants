use iced::{
    alignment,
    widget::{button, column, container, text},
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
}

impl Vault {
    pub fn new(name: String, entries: Vec<Entry>) -> Self {
        Self { name, entries }
    }

    pub fn view(&self) -> Element<VaultMessage> {
        let header = text(self.name.to_string());
        let content = container(column(self.entries.iter().map(|e| {
            e.view()
                .map(move |message| VaultMessage::Entry(message, e.key.clone()))
        })));
        let new_button = button(
            text("+")
                .horizontal_alignment(alignment::Horizontal::Center)
                .width(Length::Fill),
        )
        .width(Length::Fill)
        .on_press(VaultMessage::NewEntry);
        container(column![header, content, new_button]).into()
    }
}
