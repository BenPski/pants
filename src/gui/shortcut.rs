use iced::keyboard::{self};

use super::gui_message::GUIMessage;

pub struct Shortcut {
    base_key: keyboard::Key,
    modifier: Option<keyboard::Modifiers>,
    message: GUIMessage,
}

impl Shortcut {
    pub fn new(
        base_key: keyboard::Key,
        modifier: Option<keyboard::Modifiers>,
        message: GUIMessage,
    ) -> Self {
        Shortcut {
            base_key,
            modifier,
            message,
        }
    }

    pub fn check(&self, key: &keyboard::Key, modifier: &keyboard::Modifiers) -> Option<GUIMessage> {
        if let Some(self_modifier) = self.modifier {
            if self.base_key == *key && self_modifier == *modifier {
                Some(self.message.clone())
            } else {
                None
            }
        } else {
            if self.base_key == *key {
                Some(self.message.clone())
            } else {
                None
            }
        }
    }

    pub fn key_display(&self) -> String {
        let base_key = match self.base_key {
            keyboard::Key::Named(key) => named_display(&key).to_string(),
            keyboard::Key::Character(ref c) => c.to_string(),
            keyboard::Key::Unidentified => "unidentified".to_string(),
        };

        if let Some(modifier) = self.modifier {
            let modifier = modifier_display(&modifier);
            format!("{}+{}", modifier, base_key)
        } else {
            base_key.to_string()
        }
    }

    pub fn message(&self) -> &GUIMessage {
        &self.message
    }
}

fn modifier_display(modifier: &keyboard::Modifiers) -> &str {
    match *modifier {
        keyboard::Modifiers::SHIFT => "Shift",
        keyboard::Modifiers::ALT => "Alt",
        keyboard::Modifiers::CTRL => "Ctrl",
        keyboard::Modifiers::LOGO => "Cmd",
        _ => unimplemented!(),
    }
}

fn named_display(key: &keyboard::key::Named) -> &str {
    match *key {
        keyboard::key::Named::Tab => "Tab",
        _ => unimplemented!(),
    }
}
