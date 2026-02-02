use std::str::FromStr;
use chrono::{DateTime, TimeZone, Utc};
use regex::Regex;
use anyhow::{Result, anyhow};

#[derive(Debug, Clone, Default)]
pub struct LogEntry {
    pub ip: String,
    pub timestamp: DateTime<Utc>,
    pub method: String,
    pub path: String,
    pub status: u16,
    pub size: u64,
}

pub enum Error {
    ParseError,
}

pub fn parse_common_log(line: &str) -> Result<LogEntry> {
    // 127.0.0.1 - - [01/Jan/2024:12:00:00 +0000] "GET /api HTTP/1.1" 200 1234
    let pattern = r#"(?<ip>[[:digit:]]{1,3}\.[[:digit:]]{1,3}\.[[:digit:]]{1,3}\.[[:digit:]]{1,3}) - - \[(?<timestamp>.+)\] "(?<method>.+) (?<path>/.+) .+" (?<status>[[:digit:]]{3}) (?<size>.+)"#;
    let regex = Regex::new(pattern).unwrap();
    let Some(caps) = regex.captures(line) else { return Err(anyhow!("Error parsing line")); };

    let date_format = "%d/%b/%Y:%H:%M:%S %z";
    let timestamp: DateTime<Utc> = DateTime::parse_from_str(&caps["timestamp"], date_format)?.to_utc();

    let ip = caps["ip"].to_string();
    let method = caps["method"].to_string();
    let path = caps["path"].to_string();
    let status = caps["status"].parse::<u16>()?;
    let size = caps["size"].parse::<u64>()?;

    let entry = LogEntry { ip, timestamp, method, path, status, size };
    return Ok(entry)
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let line =  "127.0.0.1 - - [01/Jan/2024:12:00:00 +0000] \"GET /api HTTP/1.1\" 200 1234";

        let log = parse_common_log(line).unwrap();
        assert_eq!(log.ip, "127.0.0.1".to_string());
        assert_eq!(log.timestamp, Utc.with_ymd_and_hms(2024, 01, 01, 12, 0, 0).unwrap());
        assert_eq!(log.method, "GET".to_string());
        assert_eq!(log.path, "/api".to_string());
        assert_eq!(log.status, 200);
        assert_eq!(log.size, 1234);
    }
}
