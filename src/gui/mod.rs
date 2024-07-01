use iced::{Padding, Rectangle};

pub mod connection;
pub mod entry;
pub mod gui_message;
pub mod shortcut;
pub mod state;
pub mod style;
pub mod temp_message;
pub mod vault;
pub mod widget;

pub fn with_padding(rect: Rectangle, padding: Padding) -> Rectangle {
    Rectangle {
        x: rect.x - padding.left,
        y: rect.y - padding.top,
        width: rect.width + padding.left + padding.right,
        height: rect.height + padding.left + padding.right,
    }
}
