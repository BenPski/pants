use std::collections::HashMap;

use iced::Theme;

pub fn theme_map() -> HashMap<String, Theme> {
    Theme::ALL
        .iter()
        .map(|t| (t.to_string(), t.clone()))
        .collect::<HashMap<_, _>>()
}
