use std::error::Error;

use chrono::{DateTime, Local, NaiveDateTime};

pub fn now() -> DateTime<Local> {
    Local::now()
}

pub fn format_date(date: DateTime<Local>) -> String {
    date.format("%Y_%m_%d_%H_%M_%S_%f").to_string()
}

pub fn read_date(date: &str) -> std::result::Result<DateTime<Local>, Box<dyn Error>> {
    let res = NaiveDateTime::parse_from_str(date, "%Y_%m_%d_%H_%M_%S_%f")
        .map(|x| x.and_local_timezone(Local).unwrap())?;
    Ok(res)
}
