use std::path::PathBuf;

use chrono::{DateTime, NaiveDateTime, ParseError, Utc};

pub fn now() -> DateTime<Utc> {
    Utc::now()
}

pub fn format_date(date: DateTime<Utc>) -> String {
    date.format("%Y_%m_%d_%H_%M_%S_%f").to_string()
}

pub fn read_date(date: &str) -> Result<DateTime<Utc>, ParseError> {
    let res = NaiveDateTime::parse_from_str(date, "%Y_%m_%d_%H_%M_%S_%f")
        .map(|x| x.and_local_timezone(Utc).unwrap())?;
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
