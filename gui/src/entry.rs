use iced::{
    widget::{button, container, row, text},
    Element, Length,
};

#[derive(Debug, Clone)]
pub struct Entry {
    pub key: String,
}

#[derive(Debug, Clone)]
pub enum EntryMessage {
    Delete,
    View,
}

impl Entry {
    pub fn new(key: String, _style: String) -> Self {
        Entry { key }
    }

    pub fn view(&self) -> Element<EntryMessage> {
        let value = text(self.key.clone()).width(Length::Fill);
        let view_button = button("View").on_press(EntryMessage::View);
        let delete_button = button("Delete")
            .on_press(EntryMessage::Delete)
            .style(button::danger);
        let content = row![view_button, value, delete_button];
        container(content)
            .width(Length::Fill)
            .height(Length::Shrink)
            .into()
    }
}
