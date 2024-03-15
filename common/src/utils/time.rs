use chrono::prelude::*;

pub const MINUTE1: u64 = 60 * 1000;

pub const MINUTE10: u64 = 10 * MINUTE1;

pub const MINUTE30: u64 = 30 * MINUTE1;

pub const HOUR1: u64 = 60 * MINUTE1;

pub const DAY1: u64 = 24 * HOUR1;
pub const DAY15: u64 = 15 * DAY1;
//convenient for test
pub const YEAR100: u64 = 100 * 365 * DAY1;

pub fn current_date() -> String {
    let dt: DateTime<Local> = Local::now();
    dt.format("%Y-%m-%d %H:%M:%S.%f").to_string()
}

pub fn now_millis() -> u64 {
    Local::now().timestamp_millis() as u64
}

pub fn now_nanos() -> u64 {
    Local::now().timestamp_nanos_opt().unwrap() as u64
}

pub fn time2unix(time_str: String) -> u64 {
    let dt = Utc
        .datetime_from_str(time_str.as_str(), "%Y-%m-%d %H:%M:%S.%f")
        .unwrap();
    dt.timestamp_millis() as u64
}
