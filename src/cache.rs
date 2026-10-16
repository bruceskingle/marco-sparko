
use std::path::PathBuf;

use sparko_graphql::types::Date;
use time::Month;

pub struct CacheConfig {
    dir_path: PathBuf,
    pub verbose: bool,
}

/// The configuration for a monthly bucket
pub struct MonthBucket {
    /// Path to the local cache file
    pub path: PathBuf,
    /// Start date for this bucket
    pub start: Date,
    /// Start date for the next bucket
    pub next: Date,
}

impl CacheConfig {
    pub fn path_for_hash_key(&self, hash_key: &str) -> PathBuf {
        let mut path = self.dir_path.clone();
        path.push(hash_key);
        path
    }

    pub fn path_for_month(&self, hash_key: &str, date: &Date) -> anyhow::Result<MonthBucket> {
        let start = Date::from_calendar_date(date.year(), date.month(), 1)?;
        let next = if date.month() == Month::December {
            Date::from_calendar_date(date.year() + 1, Month::January, 1)?
        }
        else {
            Date::from_calendar_date(date.year(), date.month().next(), 1)?
        };

        let mut path = self.dir_path.clone();

        Self::path_for_date(&mut path, date);
        Self::path_hash_key_for_date(&mut path, date, hash_key);
        Ok( MonthBucket {path, start, next})
    }

    fn path_for_date(path: &mut PathBuf, date: &Date) {
        path.push(date.year().to_string());
        // path.push(date.month().to_string());
    }

    fn path_hash_key_for_date(path: &mut PathBuf, date: &Date, hash_key: &str) {
        // path.push(format!("{}#{}", date.day(), hash_key));
        path.push(format!("{}#{}", date.month(), hash_key));
    }
    
    pub fn new(dir_path: PathBuf, verbose: bool) -> Self {
        CacheConfig {
            dir_path,
            verbose,
        }
    }
}