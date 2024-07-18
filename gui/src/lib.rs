use std::collections::{BTreeMap, HashMap};

use gui_message::GUIMessage;
use iced::{keyboard, widget::text_input, Padding, Rectangle, Theme};
use once_cell::sync::Lazy;
use shortcut::Shortcut;
use state::manager::ManagerState;

pub mod client_config;
pub mod connection;
pub mod entry;
pub mod gui_message;
pub mod shortcut;
pub mod state;
pub mod style;
pub mod temp_message;
pub mod utils;
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

pub static THEMES: Lazy<BTreeMap<String, Theme>> = Lazy::new(|| {
    Theme::ALL
        .iter()
        .map(|t| (t.to_string(), t.clone()))
        .collect::<BTreeMap<_, _>>()
});

pub static SHORTCUTS: Lazy<HashMap<String, Shortcut>> = Lazy::new(|| {
    HashMap::from_iter(vec![
        (
            "New Vault".to_string(),
            Shortcut::new(
                keyboard::Key::Character("n".into()),
                Some(keyboard::Modifiers::COMMAND),
                GUIMessage::NewVault,
            ),
        ),
        (
            "Quit".to_string(),
            Shortcut::new(
                keyboard::Key::Character("q".into()),
                Some(keyboard::Modifiers::COMMAND),
                GUIMessage::Close,
            ),
        ),
        (
            "Tab forward".to_string(),
            Shortcut::new(
                keyboard::Key::Named(keyboard::key::Named::Tab),
                None,
                GUIMessage::TabPressed(false),
            ),
        ),
        (
            "Tab backwards".to_string(),
            Shortcut::new(
                keyboard::Key::Named(keyboard::key::Named::Tab),
                Some(keyboard::Modifiers::SHIFT),
                GUIMessage::TabPressed(true),
            ),
        ),
    ])
});

pub static INPUT_ID: Lazy<text_input::Id> = Lazy::new(text_input::Id::unique);

pub fn run() -> iced::Result {
    iced::application(
        ManagerState::title,
        ManagerState::update,
        ManagerState::view,
    )
    .subscription(ManagerState::subscription)
    .theme(ManagerState::theme)
    .run()
}
