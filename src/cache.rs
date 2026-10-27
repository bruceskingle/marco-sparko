
use std::path::PathBuf;

use sparko_graphql::types::Date;
use time::Month;

pub struct CacheConfig {
    dir_path: PathBuf,
    pub verbose: bool,
    /// Only what is cached is used, nothing is fetched, for a redacted profile, see Profile::offline.
    pub offline: bool,
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
        // The year directory does not exist the first time a month in that year is cached.
        crate::private_file::create_private_dir(&path)?;
        Self::path_hash_key_for_date(&mut path, date, hash_key);
        Ok( MonthBucket {path, start, next})
    }

/// The cache file for the day date, in a directory for its month in the directory for its year, e.g.
    /// 2026/October/25#hash_key
    pub fn path_for_day(&self, hash_key: &str, date: &Date) -> anyhow::Result<PathBuf> {
        let mut path = self.dir_path.clone();
        Self::path_for_date(&mut path, date);
        // The directories do not exist the first time a day in that year or month is cached.
        crate::private_file::create_private_dir(&path)?;
        path.push(date.month().to_string());
        crate::private_file::create_private_dir(&path)?;
        path.push(format!("{}#{}", date.day(), hash_key));
        Ok(path)
    }

    fn path_for_date(path: &mut PathBuf, date: &Date) {
        path.push(date.year().to_string());
        // path.push(date.month().to_string());
    }

    fn path_hash_key_for_date(path: &mut PathBuf, date: &Date, hash_key: &str) {
        // path.push(format!("{}#{}", date.day(), hash_key));
        path.push(format!("{}#{}", date.month(), hash_key));
    }
    
    pub fn new(dir_path: PathBuf, verbose: bool, offline: bool) -> Self {
        CacheConfig {
            dir_path,
            verbose,
            offline,
        }
    }
}