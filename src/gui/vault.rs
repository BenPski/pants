use std::collections::BTreeMap;

use iced::{
    alignment, theme,
    widget::{button, column, container, row, text},
    Element, Length, Theme,
};

use crate::schema::Schema;

use super::entry::{Entry, EntryMessage};

#[derive(Default)]
pub struct Vault {
    pub name: String,
    pub entries: BTreeMap<String, Entry>,
    pub expanded: bool,
}

#[derive(Debug, Clone)]
pub enum VaultMessage {
    Entry(EntryMessage, String),
    Toggle(String),
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

    pub fn name(mut self, value: impl Into<String>) -> Self {
        self.name = value.into();
        self
    }

    pub fn entries(mut self, value: impl Into<BTreeMap<String, Entry>>) -> Self {
        self.entries = value.into();
        self
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
        let delete_button = button("X")
            .on_press(VaultMessage::Delete)
            .style(theme::Button::Destructive);
        let header = row![name, delete_button];
        let mut entries = self
            .entries.values().map(|e| {
                e.view()
                    .map(move |message| VaultMessage::Entry(message, e.key.clone()))
            })
            .collect::<Vec<_>>();
        entries.push(
            container(
                button(
                    text("+")
                        .horizontal_alignment(alignment::Horizontal::Center)
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
        let display = if self.expanded {
            column![
                button(header).on_press(VaultMessage::Toggle(self.name.clone())),
                content
            ]
        } else {
            column![button(header).on_press(VaultMessage::Toggle(self.name.clone()))]
        };
        container(display)
            .padding(10)
            .style(|theme: &Theme| {
                let palette = theme.extended_palette();
                container::Appearance::default().with_border(palette.background.strong.color, 4.0)
            })
            .into()
    }
}
