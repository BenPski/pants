use iced::{
    alignment, theme,
    widget::{button, column, container, row, text},
    Element, Length, Theme,
};

use super::entry::{Entry, EntryMessage};

pub struct Vault {
    pub name: String,
    pub entries: Vec<Entry>,
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
    pub fn new(name: String, entries: Vec<Entry>) -> Self {
        Self {
            name,
            entries,
            expanded: false,
        }
    }

    pub fn view(&self) -> Element<VaultMessage> {
        let name = text(self.name.to_string()).size(20).width(Length::Fill);
        let delete_button = button("X")
            .on_press(VaultMessage::Delete)
            .style(theme::Button::Destructive);
        let header = row![name, delete_button];
        let mut entries = self
            .entries
            .iter()
            .map(|e| {
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
