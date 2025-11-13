use std::fs::{File, OpenOptions};
use std::io::{BufReader, Lines, Write};
use std::io::BufRead;
use std::path::Path;
use std::{fs, path::PathBuf};
use anyhow::anyhow;
use indexmap::IndexMap;
use serde::de::DeserializeOwned;
use serde::Serialize;
use fs4::fs_std::FileExt; // Import the trait for fs4 methods

use sparko_graphql::types::Date;
use time::Month;

/* ***************************************************************************************************************************************************************
 * Manager for local file system cache.
 * 
 * The current implementation uses advisory locking to prevent clashes by multiple concurrent processes, but this is imperfect because if a file has been updated
 * by another process we may end up appending records which have already been added by that other process.
 *************************************************************************************************************************************************************** */

pub struct CacheManager {
    pub dir_path: PathBuf,
    pub verbose: bool,
}

pub type Indexer<T> = Box<dyn Fn(&T) -> String + Send + Sync>;

impl CacheManager {
    fn path_for_date(path: &mut PathBuf, date: &Date) {
        path.push(date.year().to_string());
        // path.push(date.month().to_string());
    }

    fn path_hash_key_for_date(path: &mut PathBuf, date: &Date, hash_key: &str) {
        // path.push(format!("{}#{}", date.day(), hash_key));
        path.push(format!("{}#{}", date.month(), hash_key));
    }


    ////////////////
    /// 
    /// 
    
    pub fn write_vec<T: Serialize>(&self, hash_key: &str, vec: &Vec<(String, T)>, cached_cnt: usize) -> anyhow::Result<()> {
        let mut path = self.dir_path.clone();
        path.push(hash_key);

        self.do_write_vec(path, vec, cached_cnt)
    }

    pub fn write_vec_for_date<T: Serialize>(&self, date: &Date, hash_key: &str, vec: &Vec<(String, T)>, cached_cnt: usize) -> anyhow::Result<()> {
        let mut path = self.dir_path.clone();

        Self::path_for_date(&mut path, date);

        fs::create_dir_all(&path)?;

        Self::path_hash_key_for_date(&mut path, date, hash_key);

        self.do_write_vec(path, vec, cached_cnt)
    }

    fn do_write_vec<T: Serialize>(&self, path: PathBuf, vec: &Vec<(String, T)>, cached_cnt: usize) -> anyhow::Result<()> {

        if cached_cnt == 0 {
            let mut out = fs::File::create(path)?;
            for (key, value) in vec {
                writeln!(out, "{}\t{}", key, serde_json::to_string(&value)?)?;
                if self.verbose 
                {
                    println!("WRITE {}", key);
                }
            }
        }
        else {
            let mut out = OpenOptions::new().append(true).open(path)?;

            let mut i = cached_cnt;
            while i < vec.len() {
                let (key, value) = vec.get(i).unwrap();
                writeln!(out, "{}\t{}", key, serde_json::to_string(&value)?)?;
                i += 1;
            }
        }

        Ok(())
    }

pub fn read_vec<T: DeserializeOwned>(&self, hash_key: &str, vec: &mut Vec<(String, T)>) -> anyhow::Result<()> {
        let mut path = self.dir_path.clone();
        path.push(hash_key);

        self.do_read_vec(path, vec)
    }

    pub fn read_vec_for_date<T: DeserializeOwned>(&self, date: &Date, hash_key: &str, vec: &mut Vec<(String, T)>) -> anyhow::Result<(Date, Date)> {
        let start_date = Date::from_calendar_date(date.year(), date.month(), 1)?;
        let end_date = if date.month() == Month::December {
            Date::from_calendar_date(date.year() + 1, Month::January, 1)?
        }
        else {
            Date::from_calendar_date(date.year(), date.month().next(), 1)?
        };
        let mut path = self.dir_path.clone();

        Self::path_for_date(&mut path, date);
        Self::path_hash_key_for_date(&mut path, date, hash_key);

        self.do_read_vec(path, vec)?;

        Ok((start_date, end_date))
    }

    fn do_read_vec<T: DeserializeOwned>(&self, path: PathBuf, vec: &mut Vec<(String, T)>) -> anyhow::Result<()> {
        match Self::read_lines(path) {
            Ok(lines) => {
                // Consumes the iterator, returns an (Optional) String
                for line in lines.map_while(Result::ok) {
                    if self.verbose 
                    {
                        println!("READ {}", line);
                    }

                    match line.split_once('\t') {
                        Some((key, value)) => vec.push((key.to_string(), serde_json::from_str(value)?)),
                        None => return Err(anyhow!(format!("Invalid cached object <{}>", line))),
                    }
                }
            },

            Err(error) => {
                if error.kind() != std::io::ErrorKind::NotFound {
                    println!("ERROR {:?}", error);
                    return Err(anyhow!(error))
                }
                
            },
        }

        Ok(())
    }


    /// 
    /// ////////////

    pub fn write<T: Serialize>(&self, hash_key: &str, map: &IndexMap<String, (String, T)>, cached_cnt: usize) -> anyhow::Result<()> {
        let mut path = self.dir_path.clone();
        path.push(hash_key);

        self.do_write(path, map, cached_cnt)
    }

    pub fn write_for_date<T: Serialize>(&self, date: &Date, hash_key: &str, map: &IndexMap<String, (String, T)>, cached_cnt: usize) -> anyhow::Result<()> {
        let mut path = self.dir_path.clone();

        Self::path_for_date(&mut path, date);

        fs::create_dir_all(&path)?;

        Self::path_hash_key_for_date(&mut path, date, hash_key);

        self.do_write(path, map, cached_cnt)
    }

    fn do_write<T: Serialize>(&self, path: PathBuf, map: &IndexMap<String, (String, T)>, cached_cnt: usize) -> anyhow::Result<()> {

        if cached_cnt == 0 {
            let mut out = fs::File::create(path)?;
            let _guard = out.lock_exclusive()?;
            for (key, value) in map.values() {
                writeln!(out, "{}\t{}", key, serde_json::to_string(&value)?)?;
                if self.verbose 
                {
                    println!("WRITE {}", key);
                }
            }
            // drop(guard);
        }
        else {
            let mut out = OpenOptions::new().append(true).open(path)?;
            let _guard = out.lock_exclusive()?;

            let mut i = cached_cnt;
            while i < map.len() {
                let (_index, (key, value)) = map.get_index(i).unwrap();
                writeln!(out, "{}\t{}", key, serde_json::to_string(&value)?)?;
                i += 1;
            }
        }

        Ok(())
    }

    // The output is wrapped in a Result to allow matching on errors.
    // Returns an Iterator to the Reader of the lines of the file.
    fn read_lines<P>(filename: P) -> std::io::Result<Lines<BufReader<File>>>
    where P: AsRef<Path>, {
        let file = File::open(filename)?;
        let _guard = file.lock_shared()?;
        Ok(BufReader::new(file).lines())
    }

    pub fn read<T: DeserializeOwned>(&self, hash_key: &str, map: &mut IndexMap<String, (String, T)>, indexer: &Indexer<T>) -> anyhow::Result<()> {
        let mut path = self.dir_path.clone();
        path.push(hash_key);

        self.do_read(path, map, indexer)
    }

    pub fn read_for_date<T: DeserializeOwned>(&self, date: &Date, hash_key: &str, map: &mut IndexMap<String, (String, T)>, indexer: &Indexer<T>) -> anyhow::Result<(Date, Date)> {
        let start_date = Date::from_calendar_date(date.year(), date.month(), 1)?;
        let end_date = if date.month() == Month::December {
            Date::from_calendar_date(date.year() + 1, Month::January, 1)?
        }
        else {
            Date::from_calendar_date(date.year(), date.month().next(), 1)?
        };
        let mut path = self.dir_path.clone();

        Self::path_for_date(&mut path, date);
        Self::path_hash_key_for_date(&mut path, date, hash_key);

        self.do_read(path, map, indexer)?;

        Ok((start_date, end_date))
    }

    fn do_read<T: DeserializeOwned>(&self, path: PathBuf, map: &mut IndexMap<String, (String, T)>, indexer: &Indexer<T>) -> anyhow::Result<()> {
        match Self::read_lines(path) {
            Ok(lines) => {
                // Consumes the iterator, returns an (Optional) String
                for line in lines.map_while(Result::ok) {
                    if self.verbose 
                    {
                        println!("READ {}", line);
                    }

                    match line.split_once('\t') {
                        Some((key, value)) => {
                            let value = serde_json::from_str(value)?;
                            let index = indexer(&value);
                            map.insert(index, (key.to_string(), value));

                            // vec.push((key.to_string(), value))
                        },
                        None => return Err(anyhow!(format!("Invalid cached object <{}>", line))),
                    }
                }
            },

            Err(error) => {
                if error.kind() != std::io::ErrorKind::NotFound {
                    println!("ERROR {:?}", error);
                    return Err(anyhow!(error))
                }
                
            },
        }

        Ok(())
    }


    pub fn write_one<T: Serialize>(&self, hash_key: &str, value: &T) -> anyhow::Result<()> {
        let mut path = self.dir_path.clone();
        path.push(hash_key);

        let mut out = fs::File::create(path)?;
        let _guard = out.lock_exclusive()?;

        writeln!(out, "{}", serde_json::to_string(&value)?)?;

        Ok(())
    }

    pub fn read_one<T: DeserializeOwned>(&self, hash_key: &str) -> anyhow::Result<Option<T>> {
        let mut path = self.dir_path.clone();
        path.push(hash_key);

        Ok(match File::open(path) {
            Ok(file) => {
                let _guard = file.lock_shared()?;
                let reader = BufReader::new(file);
                Some(serde_json::from_reader(reader)?)
            },
            Err(error) => {
                if error.kind() != std::io::ErrorKind::NotFound {
                    println!("ERROR {:?}", error);
                    return Err(anyhow!(error))
                }
                None
            },
        })
    }
}