//! The log of important events, e.g. data fetched or requests which failed, as ERROR, WARNING or INFO entries, kept in a
//! file for each profile in the cache directory and shown in the log panel, see views/log_panel.rs. Verbose output (-v)
//! still goes to stdout and is not logged.
//!
//! The file is managed by the profile's log settings, see LogRetention: by default it only has the latest run of the
//! app, or it can keep the last few days, or be limited to a size.

use std::collections::VecDeque;
use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::sync::Mutex;
use std::sync::atomic::{AtomicU64, Ordering};

use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use time::format_description::well_known::Rfc3339;

/// The most entries held for the log panel, older ones are only in the file.
const MAX_ENTRIES: usize = 10_000;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Level {
    Error,
    Warning,
    Info,
}

impl Level {
    fn as_str(&self) -> &'static str {
        match self {
            Level::Error => "ERROR",
            Level::Warning => "WARNING",
            Level::Info => "INFO",
        }
    }

    fn parse(text: &str) -> Option<Self> {
        match text {
            "ERROR" => Some(Level::Error),
            "WARNING" => Some(Level::Warning),
            "INFO" => Some(Level::Info),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct LogEntry {
    pub time: OffsetDateTime,
    pub level: Level,
    pub message: String,
}

impl LogEntry {
    /// A line of the file, e.g. 2026-10-26T06:40:20Z INFO Fetched meterTelemetry
    fn line(&self) -> String {
        format!("{} {} {}", self.time.format(&Rfc3339).unwrap_or_default(), self.level.as_str(), self.message)
    }

    fn parse(line: &str) -> Option<Self> {
        let mut parts = line.splitn(3, ' ');
        let time = OffsetDateTime::parse(parts.next()?, &Rfc3339).ok()?;
        let level = Level::parse(parts.next()?)?;
        Some(LogEntry { time, level, message: parts.next().unwrap_or_default().to_string() })
    }
}

/// How much of the log file is kept.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "keep", rename_all = "camelCase")]
pub enum LogRetention {
    /// Only the latest run of the app, the file is started again each time it starts.
    LastRun,
    /// The entries of the last so many days.
    Days { days: u32 },
    /// At most this many megabytes, the oldest entries are removed to keep it under.
    Size { megabytes: u32 },
}

impl Default for LogRetention {
    fn default() -> Self {
        LogRetention::LastRun
    }
}

/// The log settings of a profile, see Settings.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LogSettings {
    #[serde(flatten)]
    pub retention: LogRetention,
    /// Whether the log panel is shown.
    #[serde(default = "shown")]
    pub show: bool,
}

fn shown() -> bool {
    true
}

impl Default for LogSettings {
    fn default() -> Self {
        LogSettings { retention: LogRetention::default(), show: true }
    }
}

struct Logger {
    path: PathBuf,
    file: Option<File>,
    retention: LogRetention,
    size: u64,
    entries: VecDeque<LogEntry>,
}

static LOGGER: Mutex<Option<Logger>> = Mutex::new(None);
/// Changes whenever an entry is added or the log is started, so that the log panel knows when to show it again.
static GENERATION: AtomicU64 = AtomicU64::new(0);

/// The log file of a profile, in the cache directory.
pub fn log_path(profile_name: &str) -> anyhow::Result<PathBuf> {
    Ok(crate::MarcoSparkoContext::get_cache_dir_path()?.join(format!("{profile_name}.log")))
}

/// The entries of the file which are kept by retention, oldest first.
fn kept(lines: &str, retention: LogRetention, now: OffsetDateTime) -> Vec<LogEntry> {
    let entries = lines.lines().filter_map(LogEntry::parse);
    match retention {
        LogRetention::LastRun => Vec::new(),
        LogRetention::Days { days } => {
            let from = now - time::Duration::days(days as i64);
            entries.filter(|entry| entry.time >= from).collect()
        },
        LogRetention::Size { megabytes } => {
            // The newest entries, up to half the size, so that it is not trimmed again with every entry.
            let limit = megabytes as u64 * 1024 * 1024 / 2;
            let mut size = 0;
            let mut kept: Vec<LogEntry> = entries.collect::<Vec<_>>().into_iter().rev()
                .take_while(|entry| { size += entry.line().len() as u64 + 1; size <= limit })
                .collect();
            kept.reverse();
            kept
        },
    }
}

/// Writes the entries to the file at path, replacing what is there, and returns it open to add to, and its size.
fn rewrite(path: &PathBuf, entries: &[LogEntry]) -> std::io::Result<(File, u64)> {
    let mut text = String::new();
    for entry in entries {
        text.push_str(&entry.line());
        text.push('\n');
    }
    fs::write(path, &text)?;
    let file = OpenOptions::new().append(true).open(path)?;
    Ok((file, text.len() as u64))
}

/// Starts the log of the profile, with the entries the settings keep from before, then logs that the app started. If
/// the file can not be written the entries are still shown in the log panel. Called when the profile is chosen.
pub fn start(profile_name: &str, settings: &LogSettings, started: &str) {
    let path = match log_path(profile_name) {
        Ok(path) => path,
        Err(error) => {
            eprintln!("Unable to find the log file: {error}");
            return;
        },
    };
    let previous = fs::read_to_string(&path).unwrap_or_default();
    let entries = kept(&previous, settings.retention, OffsetDateTime::now_utc());
    let (file, size) = match rewrite(&path, &entries) {
        Ok((file, size)) => (Some(file), size),
        Err(error) => {
            eprintln!("Unable to write the log file {}: {error}", path.display());
            (None, 0)
        },
    };
    let start = entries.len().saturating_sub(MAX_ENTRIES);
    *LOGGER.lock().unwrap() = Some(Logger {
        path,
        file,
        retention: settings.retention,
        size,
        entries: entries.into_iter().skip(start).collect(),
    });
    GENERATION.fetch_add(1, Ordering::SeqCst);
    log(Level::Info, format!("{started}, Marco Sparko {}, profile {profile_name}", env!("CARGO_PKG_VERSION")));
}

/// Changes how much of the log is kept, from now on. A size limit is applied at once, a number of days and the last run
/// only the next time the app starts, so that this run's entries are not lost.
pub fn set_retention(retention: LogRetention) {
    let mut logger = LOGGER.lock().unwrap();
    if let Some(logger) = logger.as_mut() {
        logger.retention = retention;
        trim(logger);
    }
}

/// Removes the oldest entries from the file if it is over its size limit.
fn trim(logger: &mut Logger) {
    if let LogRetention::Size { megabytes } = logger.retention
        && logger.size > megabytes as u64 * 1024 * 1024
    {
        let entries = kept(&fs::read_to_string(&logger.path).unwrap_or_default(), logger.retention, OffsetDateTime::now_utc());
        if let Ok((file, size)) = rewrite(&logger.path, &entries) {
            logger.file = Some(file);
            logger.size = size;
        }
    }
}

/// Adds an entry to the log, see the log_info, log_warning and log_error macros. Newlines in the message are replaced
/// so that each entry is one line of the file.
pub fn log(level: Level, message: String) {
    // To the second, which is all that is useful to read.
    let time = OffsetDateTime::now_utc();
    let time = time.replace_nanosecond(0).unwrap_or(time);
    let entry = LogEntry { time, level, message: message.replace(['\r', '\n'], " ") };
    let mut logger = LOGGER.lock().unwrap();
    // Before the log is started, e.g. while the profile is read, entries are only printed.
    let Some(logger) = logger.as_mut() else {
        println!("{}", entry.line());
        return;
    };
    let line = entry.line();
    if let Some(file) = logger.file.as_mut()
        && writeln!(file, "{line}").is_ok()
    {
        logger.size += line.len() as u64 + 1;
        trim(logger);
    }
    logger.entries.push_back(entry);
    if logger.entries.len() > MAX_ENTRIES {
        logger.entries.pop_front();
    }
    GENERATION.fetch_add(1, Ordering::SeqCst);
}

/// A number which changes whenever the log does.
pub fn generation() -> u64 {
    GENERATION.load(Ordering::SeqCst)
}

/// The entries held for the log panel, oldest first.
pub fn entries() -> Vec<LogEntry> {
    LOGGER.lock().unwrap().as_ref().map(|logger| logger.entries.iter().cloned().collect()).unwrap_or_default()
}

/// Logs an INFO entry, e.g. log_info!("Fetched {count} readings").
#[macro_export]
macro_rules! log_info {
    ($($arg:tt)*) => { $crate::logging::log($crate::logging::Level::Info, format!($($arg)*)) };
}

/// Logs a WARNING entry, e.g. a request which is being retried.
#[macro_export]
macro_rules! log_warning {
    ($($arg:tt)*) => { $crate::logging::log($crate::logging::Level::Warning, format!($($arg)*)) };
}

/// Logs an ERROR entry, e.g. a request which failed.
#[macro_export]
macro_rules! log_error {
    ($($arg:tt)*) => { $crate::logging::log($crate::logging::Level::Error, format!($($arg)*)) };
}

#[cfg(test)]
mod tests {
    use time::macros::datetime;

    use super::{Level, LogEntry, LogRetention, LogSettings, kept};

    fn line(time: time::OffsetDateTime, message: &str) -> String {
        LogEntry { time, level: Level::Info, message: message.to_string() }.line()
    }

    #[test]
    fn entries_are_kept_for_the_last_run_days_or_size() {
        let now = datetime!(2026-10-26 12:00 UTC);
        let lines = [
            line(datetime!(2026-10-20 12:00 UTC), "old"),
            line(datetime!(2026-10-25 12:00 UTC), "yesterday"),
            line(now, "now"),
        ].join("\n");
        let messages = |retention| kept(&lines, retention, now).into_iter().map(|entry| entry.message).collect::<Vec<_>>();
        assert!(messages(LogRetention::LastRun).is_empty());
        assert_eq!(messages(LogRetention::Days { days: 2 }), ["yesterday", "now"]);
        assert_eq!(messages(LogRetention::Size { megabytes: 1 }), ["old", "yesterday", "now"]);
    }

    #[test]
    fn a_line_is_read_back_as_it_was_written() {
        let entry = LogEntry { time: datetime!(2026-10-26 06:40:20 UTC), level: Level::Warning, message: String::from("Too many requests") };
        assert_eq!(entry.line(), "2026-10-26T06:40:20Z WARNING Too many requests");
        assert_eq!(LogEntry::parse(&entry.line()), Some(entry));
    }

    #[test]
    fn settings_are_stored_with_the_retention_flattened() {
        let settings = LogSettings { retention: LogRetention::Days { days: 7 }, show: false };
        let json = serde_json::to_value(&settings).unwrap();
        assert_eq!(json, serde_json::json!({ "keep": "days", "days": 7, "show": false }));
        assert_eq!(serde_json::from_value::<LogSettings>(serde_json::json!({})).ok(), None);
        assert_eq!(serde_json::from_value::<LogSettings>(serde_json::json!({ "keep": "lastRun" })).unwrap(), LogSettings::default());
    }
}
