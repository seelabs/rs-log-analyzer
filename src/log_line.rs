use lazy_static::lazy_static;
use regex::Regex;

use std::io::Write;

#[derive(Clone, Copy, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warning,
    Error,
    Fatal,
}

impl LogLevel {
    fn new(field: &str) -> Self {
        match field {
            "TRC" => LogLevel::Trace,
            "DBG" => LogLevel::Debug,
            "NFO" => LogLevel::Info,
            "WRN" => LogLevel::Warning,
            "ERR" => LogLevel::Error,
            "FTL" => LogLevel::Fatal,
            _ => panic!("Bad log level: {}", field),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub struct LogLine<'a> {
    // Declaration order is important for sorting.
    pub level: LogLevel,
    pub module: &'a str,
    pub msg: &'a str,
    pub json_data: &'a str,
    pub timestamp: &'a str,
    pub line: &'a str, // raw line from the log
}

impl<'a> LogLine<'a> {
    // Sample log line:
    // 2021-Feb-05 13:52:54.660065778 UTC TaggedCache:DBG LedgerCache target age set to 180000000000
    // Sample log line for structured log lines:
    // 2021-Feb-12 03:00:04.020060136 UTC LoadMonitor:WRN Job latency {"job": "TransactionAcquire", "run(ms)": 0, "wait(ms)": 1366, "jlogId": 115}

    pub fn new(line: &'a str) -> Option<Self> {
        lazy_static! {
            static ref UNSTRUCTURED_RE: Regex = Regex::new(
                r"(?x)
                # The x flag enables insignificant whitespace mode (allowing comments)
                ^(?P<timestamp>.*UTC)
                [\ ]
                (?P<module>[^:]*):(?P<level>[^\ ]*)
                [\ ]
                (?P<msg>.*$)
"
            )
            .unwrap();
        }

        lazy_static! {
            static ref STRUCTURED_RE: Regex = Regex::new(
                r"(?x)
                # The x flag enables insignificant whitespace mode (allowing comments)
                ^(?P<timestamp>.*UTC)
                [\ ]
                (?P<module>[^:]*):(?P<level>[^\ ]*)
                [\ ]
                (?P<msg>[^{]*)
                [\ ]
                (?P<json_data>.*$)
"
            )
            .unwrap();
        }

        let is_structured = line.ends_with("}");

        let caps = if is_structured {
            STRUCTURED_RE.captures(line)?
        } else {
            UNSTRUCTURED_RE.captures(line)?
        };

        let timestamp = caps.name("timestamp").unwrap().as_str();
        let level = LogLevel::new(caps.name("level").unwrap().as_str());
        let module = caps.name("module").unwrap().as_str();
        let msg = caps.name("msg").unwrap().as_str();
        let json_data = if is_structured {
            caps.name("json_data").unwrap().as_str()
        } else {
            ""
        };

        Some(LogLine {
            timestamp,
            level,
            module,
            msg,
            json_data,
            line,
        })
    }

    pub fn data_to_json_value(&self) -> Option<serde_json::Value> {
        if !self.json_data.is_empty() {
            if let Ok(jv) = serde_json::from_str::<serde_json::Value>(self.json_data) {
                return Some(jv);
            }
        }
        return Option::None;
    }

    pub fn to_json_value(&self) -> Option<serde_json::Value> {
        let to_jval = |s: &str| -> serde_json::Value { serde_json::Value::String(s.to_string()) };
        let mut v = serde_json::json!({});
        v["timestamp"] = to_jval(self.timestamp);
        v["module"] = to_jval(self.module);
        v["level"] = serde_json::Value::String(format!("{:?}", self.level));
        v["msg"] = to_jval(self.msg);
        if !self.json_data.is_empty() {
            if let Some(json_data) = self.data_to_json_value() {
                v["data"] = json_data;
            } else {
                return Option::None;
            }
        }

        Some(v)
    }

    // Return true if all data was written
    pub fn write_mixed_json(&self, out_file: &mut std::fs::File) -> bool {
        write!(
            out_file,
            "{} {}:{:?} {}",
            self.timestamp, self.module, self.level, self.msg
        )
        .unwrap();
        if let Some(v) = self.data_to_json_value() {
            write!(out_file, "\n{}", serde_json::to_string_pretty(&v).unwrap()).unwrap();
        } else if !self.json_data.is_empty() {
            return false;
        }
        return true;
    }
}
