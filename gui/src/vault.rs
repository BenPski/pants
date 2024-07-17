use std::collections::BTreeMap;

use boring_derive::Builder;
use iced::{
    alignment,
    widget::{button, column, container, row, text, tooltip},
    Element, Length,
};
use pants_store::schema::Schema;

use super::{
    entry::{Entry, EntryMessage},
    widget::expand::Expand,
};

#[derive(Default, Builder)]
pub struct Vault {
    pub name: String,
    pub entries: BTreeMap<String, Entry>,
    #[builder(skip)]
    pub expanded: bool,
}

#[derive(Debug, Clone)]
pub enum VaultMessage {
    Entry(EntryMessage, String),
    Toggle,
    NewEntry,
    Delete,
}

impl Vault {
    pub fn new(name: String, entries: BTreeMap<String, Entry>) -> Self {
        Self {
            name,
            entries,
            expanded: false,
        }
    }

    pub fn toggle(&mut self) {
        self.expanded = !self.expanded;
    }

    pub fn update(&mut self, schema: &Schema) {
        self.entries = schema
            .data
            .iter()
            .map(|(key, value)| {
                (
                    key.to_string(),
                    Entry::new(key.to_string(), value.to_string()),
                )
            })
            .collect();
    }

    pub fn view(&self) -> Element<VaultMessage> {
        let name = text(self.name.to_string()).size(20).width(Length::Fill);
        let delete_button = tooltip(
            button("X")
                .on_press(VaultMessage::Delete)
                .style(button::danger),
            "Delete vault",
            tooltip::Position::Bottom,
        );
        // let symbol = if self.expanded {
        //     text("- ")
        // } else {
        //     text("+ ")
        // }
        // .vertical_alignment(alignment::Vertical::Center)
        // .font(Font::MONOSPACE)
        // .width(Length::Shrink);
        let header = row![name, delete_button];
        let mut entries = self
            .entries
            .values()
            .map(|e| {
                e.view()
                    .map(move |message| VaultMessage::Entry(message, e.key.clone()))
            })
            .collect::<Vec<_>>();
        entries.push(
            container(
                button(
                    text("+")
                        .align_x(alignment::Horizontal::Center)
                        .width(Length::Fill),
                )
                .width(Length::Fill)
                .height(Length::Shrink)
                .on_press(VaultMessage::NewEntry),
            )
            .padding([10, 0])
            .height(Length::Shrink)
            .into(),
        );
        let content = container(column(entries)).padding(10);

        Expand::new(header, content, self.expanded)
            .on_press(VaultMessage::Toggle)
            .into()
    }
}
