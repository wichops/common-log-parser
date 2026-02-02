use anyhow::Result;
use chrono::{DateTime, TimeZone, Utc};
use regex::Regex;
use thiserror::Error;

#[derive(Debug, Clone, Default)]
pub struct LogEntry {
    pub ip: String,
    pub timestamp: DateTime<Utc>,
    pub method: String,
    pub path: String,
    pub status: u16,
    pub size: u64,
}

#[derive(Error, Debug, PartialEq)]
pub enum ParseError {
    #[error("Invalid log format")]
    InvalidFormat,

    #[error("Invalid timestamp ")]
    InvalidTimestamp,

    #[error("Invalid status code")]
    InvalidStatus,

    #[error("Invalid size")]
    InvalidSize,
}

pub fn parse_common_log(line: &str) -> Result<LogEntry, ParseError> {
    // 127.0.0.1 - - [01/Jan/2024:12:00:00 +0000] "GET /api HTTP/1.1" 200 1234
    let pattern = r#"(?<ip>[[:digit:]]{1,3}\.[[:digit:]]{1,3}\.[[:digit:]]{1,3}\.[[:digit:]]{1,3}) - - \[(?<timestamp>.+)\] "(?<method>.+) (?<path>/.+) .+" (?<status>[[:digit:]]{3}) (?<size>.+)"#;
    let regex = Regex::new(pattern).unwrap();
    let caps = regex.captures(line).ok_or(ParseError::InvalidFormat)?;

    let date_format = "%d/%b/%Y:%H:%M:%S %z";
    let timestamp: DateTime<Utc> = DateTime::parse_from_str(&caps["timestamp"], date_format)
        .map_err(|_| ParseError::InvalidTimestamp)?
        .to_utc();

    let ip = caps["ip"].to_string();
    let method = caps["method"].to_string();
    let path = caps["path"].to_string();
    let status = caps["status"]
        .parse::<u16>()
        .map_err(|_| ParseError::InvalidStatus)?;
    let size = caps["size"]
        .parse::<u64>()
        .map_err(|_| ParseError::InvalidSize)?;

    let entry = LogEntry {
        ip,
        timestamp,
        method,
        path,
        status,
        size,
    };
    Ok(entry)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_it_works() {
        let line = "127.0.0.1 - - [01/Jan/2024:12:00:00 +0000] \"GET /api HTTP/1.1\" 200 1234";

        let log = parse_common_log(line).unwrap();
        assert_eq!(log.ip, "127.0.0.1".to_string());
        assert_eq!(
            log.timestamp,
            Utc.with_ymd_and_hms(2024, 01, 01, 12, 0, 0).unwrap()
        );
        assert_eq!(log.method, "GET".to_string());
        assert_eq!(log.path, "/api".to_string());
        assert_eq!(log.status, 200);
        assert_eq!(log.size, 1234);
    }

    #[test]
    fn test_malformed_logs() {
        pub const MALFORMED_LOGS: [&str; 3] = [
            "invalid line with improper formatting",
            "incomplete line [15/Jan/2024:10:24:00 +0000]",
            "- - - [15/Jan/2024:10:25:00 +0000] \"GET /no-ip HTTP/1.1\" 200 50"
        ];
        for line in MALFORMED_LOGS.iter() {
            let log = parse_common_log(line);
            assert!(matches!(log, Err(ParseError::InvalidFormat)), "Wrong result in entry: {line}");
        }
    }

    #[test]
    fn test_invalid_date() {
        const INVALID_DATES: [&str; 3] = [
            "192.168.1.1 - - [15-Jan-24:10:23:45 +0000] \"GET /api/users HTTP/1.1\" 200 1234",
            "10.0.0.5 - - [5/1/2024:10:24:12 +0000] \"POST /api/login HTTP/1.1\" 201 567",
            "172.16.0.10 - - [15/Jan/2024:10:25:33] \"GET /static/image.png HTTP/1.1\" 304 0",
        ];

        for line in INVALID_DATES.iter() {
            let log = parse_common_log(line);
            assert!(matches!(log, Err(ParseError::InvalidTimestamp)));
        }
    }

    #[test]
    fn test_edge_cases() {
        let valid_logs =[
            "10.0.0.5 - - [15/Jan/2024:10:24:12 +0000] \"POST /api/login HTTP/1.1\" 201 567",
            "203.0.113.42 - - [15/Jan/2024:10:27:15 +0000] \"PUT /api/products HTTP/1.1\" 500 2048",
            "8.8.8.8 - - [15/Jan/2024:10:29:47 +0000] \"DELETE /users HTTP/1.1\" 403 89",
        ];
        let expected = [
            LogEntry {
                ip: "10.0.0.5".to_string(),
                timestamp: Utc.with_ymd_and_hms(2024, 01, 15, 10, 24, 12).unwrap(),
                method: "POST".to_string(),
                path: "/api/login".to_string(),
                status: 201,
                size: 567
            },
            LogEntry {
                ip: "203.0.113.42".to_string(),
                timestamp: Utc.with_ymd_and_hms(2024, 01, 15, 10, 27, 15).unwrap(),
                method: "PUT".to_string(),
                path: "/api/products".to_string(),
                status: 500,
                size: 2048
            },
            LogEntry {
                ip: "8.8.8.8".to_string(),
                timestamp: Utc.with_ymd_and_hms(2024, 01, 15, 10, 29, 47).unwrap(),
                method: "DELETE".to_string(),
                path: "/users".to_string(),
                status: 403,
                size: 89
            }
        ];

        for (line, expected) in valid_logs.iter().zip(expected.iter()) {
            let log = parse_common_log(line).unwrap();

            assert_eq!(log.ip, expected.ip);
            assert_eq!(log.timestamp, expected.timestamp);
            assert_eq!(log.method, expected.method);
            assert_eq!(log.path, expected.path);
            assert_eq!(log.status, expected.status);
            assert_eq!(log.size, expected.size);
        }
    }
}
