use std::{collections::HashMap, path::PathBuf};

use chrono::{DateTime, Local, NaiveDateTime, ParseError};
use iced::Theme;

pub fn now() -> DateTime<Local> {
    Local::now()
}

pub fn format_date(date: DateTime<Local>) -> String {
    date.format("%Y_%m_%d_%H_%M_%S_%f").to_string()
}

pub fn read_date(date: &str) -> Result<DateTime<Local>, ParseError> {
    let res = NaiveDateTime::parse_from_str(date, "%Y_%m_%d_%H_%M_%S_%f")
        .map(|x| x.and_local_timezone(Local).unwrap())?;
    Ok(res)
}

pub fn base_path() -> PathBuf {
    let base_dir =
        if let Some(project_dirs) = directories_next::ProjectDirs::from("com", "bski", "pants") {
            project_dirs.data_dir().into()
        } else {
            std::env::current_dir().unwrap_or_default()
        };
    base_dir
}

pub fn theme_map() -> HashMap<String, Theme> {
    Theme::ALL
        .iter()
        .map(|t| (t.to_string(), t.clone()))
        .collect::<HashMap<_, _>>()
}
