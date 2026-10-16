use std::{
    collections::{BTreeMap, HashMap}, fs::{File, FileTimes, OpenOptions}, io::{BufRead, BufReader, Seek, SeekFrom, Write}, path::PathBuf, sync::Arc, time::{Duration, SystemTime, UNIX_EPOCH},
};

use serde::{Serialize, de::DeserializeOwned};
use sparko_graphql::{
    AuthenticatedRequestManager, GraphQLQuery, GraphQLResponse,
    types::{Date, DateTime},
};
use time_tz::Tz;

use crate::{CacheConfig, octopus::token::OctopusTokenManager};

use super::DataSetAttributes;



/*
<V, R, Q, IQP, CQP, RI, IN, I, HP>
where
        V: DeserializeOwned + Serialize + Clone,
        R: GraphQLResponse,
        Q: GraphQLQuery<R>,
        IQP: FnOnce(&Date, Option<&V>) -> Result<Q, sparko_graphql::Error>,
        CQP: Fn(&Date, &R) -> Option<Result<Q, sparko_graphql::Error>>,
        RI: Fn(R) -> I,
        IN: Fn(&V) -> (DateTime, DateTime, bool),
        I: IntoIterator<Item = V>,
        HP: Fn(&Date) -> String,
*/

pub trait SlotSource {
    type Value: DeserializeOwned + Serialize + Clone;
    type Response: GraphQLResponse;
    type Query: GraphQLQuery<Self::Response>;

    fn hash_key(&self, bucket_start: &Date) -> String;
    fn initial_query(&self, start: &DateTime) -> Result<Self::Query, sparko_graphql::Error>;
    fn continuation_query(&self, start: &DateTime, resp: &Self::Response) -> Option<Result<Self::Query, sparko_graphql::Error>>;
    fn items(&self, response: Self::Response) -> impl IntoIterator<Item = Self::Value>;
    fn index(&self, value: &Self::Value) -> (DateTime, DateTime, bool);
    fn refresh_after(&self) -> Duration;
}


pub struct Item<V> {
    pub end: DateTime,
    pub is_final: bool,
    pub attributes: DataSetAttributes,
    pub value: V,
}

/// A monthly dataset of fixed-interval records that may arrive out of order.
/// Each sample has a start and end DateTime, for contiguous samples A and B A.end == B.start.
pub struct MonthlyFixedSlotDataSet<S: SlotSource>
// where
//     V: DeserializeOwned + Serialize + Clone,
{
    /// The start date of this DataSet
    pub start_date: Date,
    /// The start date of the next monthly bucket, also the exclusive end date of this one.
    pub next_date: Date,
    /// The values indexed by the start DateTime of the sample.
    pub data: BTreeMap<DateTime, Item<<S as SlotSource>::Value>>,
    /// The start DateTime of the last final sample in this DataSet
    pub last_start: DateTime,
    /// The end DateTime of the last final sample in this DataSet
    pub last_end: DateTime,
    /// True iff all samples for this bucket are. present and final.
    pub is_complete: bool,
}

impl<S: SlotSource> MonthlyFixedSlotDataSet<S> 
// where
//     V: DeserializeOwned + Serialize + Clone,
{
    fn new(
        start_date: Date,
        next_date: Date,
        tz: &'static Tz,
        data: BTreeMap<DateTime, Item<<S as SlotSource>::Value>>,
    ) -> Self {


        let mut last_end = start_date.at_midnight(tz);
        let mut last_start = last_end.clone();

        for (start, item) in &data {
            if &last_end != start {
                // there is a gap here
                break;
            }

            if item.is_final {
                last_start = start.clone();
                last_end = item.end.clone();
            }
        }

        let is_complete = last_end == next_date.at_midnight(tz);

        Self {
            start_date,
            next_date,
            data,
            last_start,
            last_end,
            is_complete
        }
    }

    fn load(
        source: &S,
        config: &CacheConfig,
        file: &mut File,
        start_date: Date,
        next_date: Date,
        // indexer: IN,
        tz: &'static Tz,
    ) -> anyhow::Result<Self>
    // where 
    //     IN: Fn(&V) -> (DateTime, DateTime, bool),
    {
        let mut data: BTreeMap<DateTime, Item<<S as SlotSource>::Value>> = BTreeMap::new();
        let lines = BufReader::new(&mut *file).lines();
        for line in lines.map_while(Result::ok) {
            if config.verbose 
            {
                println!("READ {}", line);
            }

            match serde_json::from_str(&line) {
                Ok(value) => {
                    
                    let (start, end, is_final) = source.index(&value);
                    let item = Item {
                        end,
                        is_final,
                        attributes: DataSetAttributes { cached: true, },
                        value,
                    };

                    data.insert(start, item);

                },
                Err(e) => {
                    println!("ERROR: failed to read record {:?}", &e);

                    file.seek(SeekFrom::Start(0))?;
                    file.set_len(0)?;
                    data.clear();
                    return Err(e.into());
                },
            }
        }

        Ok(Self::new(start_date, next_date, tz, data))
    }

    fn key_for(start: &Date) -> i32 {
        let key = start.year() * 12 + start.month() as i32 - 1;
        println!("Key for {:?} is {}", &start, key);
        key
    }

    pub async fn fetch(
        source: S,
        hash_key: String,
        tz: &'static Tz,
        config: &CacheConfig,
        request_manager: &AuthenticatedRequestManager<OctopusTokenManager>,
        start: &DateTime,
        end: &Option<DateTime>
    ) -> anyhow::Result<Vec<MonthlyFixedSlotDataSet<S>>>
    {
        let mut result_keys = Vec::new();
        let mut result = Vec::new();
        let mut data_set_map = HashMap::new();

        {
            // scope this start_date and end_date
            let mut start_date = start.to_date();
            let end_date = if let Some(end) = end { end.to_date() } else {Date::today(tz)};

            while start_date <= end_date {
                println!("Load cache for {}", &start_date);
                let open_data_set: OpenMonthlyFixedSlotDataSet<S> = OpenMonthlyFixedSlotDataSet::new(&source, &config, &hash_key, &start_date, tz)?;
                
                let key = Self::key_for(&start_date);
                result_keys.push(key);


                start_date = open_data_set.data_set.next_date.clone();
                data_set_map.insert(key, open_data_set);

            }
        }

        fn all_complete<S: SlotSource>(result_keys: &Vec<i32>, data_set_map: &HashMap<i32, OpenMonthlyFixedSlotDataSet<S>>) -> bool {
            for key in result_keys {
                let ds = &data_set_map.get(key).unwrap().data_set;
                if ! ds.is_complete {
                    return false;
                }
            }
            true
        }

        // At this point we have loaded all the required DataSets from local cache, if they are all complete or withing their refresh
        // duration then we can just return what we have.
        // This function returns the first missing start time unless all buckets are within their refresh duration
        fn all_up_to_date<S: SlotSource>(refresh_after: &Duration, result_keys: &Vec<i32>, data_set_map: &HashMap<i32, OpenMonthlyFixedSlotDataSet<S>>) -> anyhow::Result<Option<DateTime>> {
            let mut result = None;
            for key in result_keys {
                let ods = data_set_map.get(key).unwrap();
                let ds = &ods.data_set;
                if ! ds.is_complete {
                    if result.is_none() {
                        result = Some(ds.last_end);
                    }
                    if ods.is_stale(refresh_after)? {
                        return Ok(result);
                    }
                }
            }
            Ok(None)
        }

        let refresh_after = source.refresh_after();


        if let Some(refresh_start) = all_up_to_date(&refresh_after, &result_keys, &data_set_map)? {
            let query = source.initial_query(&refresh_start)?;
            let mut response = request_manager.call(&query).await?;

            loop {
                let cq = source.continuation_query(&refresh_start, &response);

                // Self::handle_records(response, &response_iterator, &indexer, &mut data, &mut file)?;

                for value in source.items(response) {
                    let (start, end, is_final) = source.index(&value);
                    let start_date = start.to_date();
                    let key = Self::key_for(&start_date);

                    println!("data_set_map");
                    for(key, value) in &data_set_map {
                        println!("  {} {:?}->{:?}", key, &value.data_set.start_date, &value.data_set.next_date)
                    }

                    let data_set = if let Some(data_set) = data_set_map.get_mut(&key) {
                        data_set
                    }
                    else {
                        data_set_map.insert(key, OpenMonthlyFixedSlotDataSet::new(&source, &config, &hash_key, &start_date, tz)?);
                        data_set_map.get_mut(&key).unwrap()
                    };

                    let update = if let Some(item) = data_set.data_set.data.get(&start) {
                        let v = serde_json::to_string(&value)?;
                        let c = serde_json::to_string(&item.value)?;
                        println!("COMPARE \n{}\n{}\n={}", v, c, v!=c);
                        v != c
                    }
                    else {
                        true
                    };

                    if update {
                        data_set.data_set.data.insert(start, Item {
                                end,
                                is_final,
                                attributes:DataSetAttributes { cached: false},
                                value,
                            }
                        );
                        data_set.is_dirty = true;
                    }
                }

                if all_complete(&result_keys, &data_set_map) {
                    break;
                }

                if let Some(continuation_query) = cq {
                    let query = continuation_query?;

                    response = request_manager.call(&query).await?;

                    
                }
                else {
                    break;
                }

            }
        }

        for key in result_keys {
            let ods = data_set_map.remove(&key).unwrap();

            result.push(ods.flush(&config, false)?);
        }

        for (key, ods) in data_set_map {
            ods.flush(&config, true)?;
        }

        Ok(result)
    }
}

struct OpenMonthlyFixedSlotDataSet<S: SlotSource>
{
    file: File,   // File is exclusive locked
    path: PathBuf,
    is_dirty: bool,  // Cache needs updating
    data_set: MonthlyFixedSlotDataSet<S>
}

impl<S: SlotSource> OpenMonthlyFixedSlotDataSet<S>
{
    /// Create a DataSet and load any cached value leaving the cache file locked.
    fn new(
        source: &S,
        config: &CacheConfig,
        hash_key: &str,
        date: &Date,
        tz: &'static Tz,
    ) -> anyhow::Result<Self> 
    {
        let month_bucket = config.path_for_month(hash_key, date)?;
        println!("Open \"{:?}\"", &month_bucket.path);
        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(&month_bucket.path)?;
        file.lock()?;

        let data_set: MonthlyFixedSlotDataSet<S> = MonthlyFixedSlotDataSet::load(source, config, &mut file, month_bucket.start, month_bucket.next, tz)?;
        let path = month_bucket.path;

        Ok(Self{
            file,
            path,
            is_dirty: false,
            data_set,
        })
    }

    fn is_stale(&self, refresh_after: &Duration) -> anyhow::Result<bool> {
        let modified = self.file.metadata()?.modified()?;
        Ok(SystemTime::now()
            .duration_since(modified)
            .map(|age| &age > refresh_after)
            .unwrap_or(false) // If the file is modified in the future, do not refresh it.
        )
    }
    
    fn flush(mut self, config: &CacheConfig, is_stale: bool) -> anyhow::Result<MonthlyFixedSlotDataSet<S>> {
        if self.is_dirty {
            // We added data so update the cache

            if config.verbose 
            {
                println!("REWRITING FILE {}", self.path.display());
            }
            self.file.seek(SeekFrom::Start(0))?;
            self.file.set_len(0)?;
            for (_k, item) in &self.data_set.data {
                writeln!(self.file, "{}", serde_json::to_string(&item.value)?)?;
            }

            if is_stale {
                // We fetched some data for this month but we dont know if more data might be available so we set the file modify date
                // to the epoch so that if it is read later it will be treated as stale.
                self.file.set_times(FileTimes::new().set_modified(UNIX_EPOCH))?;
            }
        }

        Ok(self.data_set)
    }
// }



// /// The manger is responsible for reading and writing individual data sets. The fetch method returns a single owned MonthlyFixedSlotDataSet
// /// but it keeps a cache of multiple sets so that when reading values which fall after the requested set can be efficiently cached.


// pub struct MonthlyFixedSlotDataSetManager<S: SlotSource> {
//     source: S,
//     hash_key: String,
//     tz: &'static time_tz::Tz,
//     config: Arc<CacheConfig>,
//     request_manager: Arc<AuthenticatedRequestManager<OctopusTokenManager>>,
// }
// // pub struct MonthlyFixedSlotDataSetManager<IQP, CQP, RI, IN, HP>
// //     // where
// //     //     V: DeserializeOwned + Serialize + Clone,
// //     //     R: GraphQLResponse,
// //     //     Q: GraphQLQuery<R>,
// //     //     IQP: FnOnce(&Date, Option<&V>) -> Result<Q, sparko_graphql::Error>,
// //     //     CQP: Fn(&Date, &R) -> Option<Result<Q, sparko_graphql::Error>>,
// //     //     RI: Fn(R) -> I,
// //     //     IN: Fn(&V) -> (DateTime, DateTime),
// //     //     I: IntoIterator<Item = V>,
// //     //     HP: Fn(&Date) -> String,
// //     {
// //         hash_key_provider: HP,
// //         timezone: &'static time_tz::Tz,
// //         initial_query_provider: IQP,
// //         continuation_query_provider: CQP,
// //         response_iterator: RI,
// //         indexer: IN,
// //         config: Arc<CacheConfig>,
// //         request_manager: Arc<AuthenticatedRequestManager<OctopusTokenManager>>,
// //         // map: HashMap<String, MonthlyFixedSlotDataSet<V>>
// // }

// // impl<IQP, CQP, RI, IN, HP> MonthlyFixedSlotDataSetManager<IQP, CQP, RI, IN, HP>


// impl<S: SlotSource> MonthlyFixedSlotDataSetManager<S> {
// //     pub fn new(source: S, /* ... */) -> anyhow::Result<Self> { /* ... */ }
// //     pub async fn fetch(&self, start: &DateTime, end: Option<&DateTime>)
// //         -> anyhow::Result<Vec<MonthlyFixedSlotDataSet<S::Value>>> { /* ... */ }
// // }
// // {
//     pub fn new
//     (
//         source: S,
//         hash_key: String,
//         tz: &'static Tz,
//         config: &Arc<CacheConfig>,
//         request_manager: &Arc<AuthenticatedRequestManager<OctopusTokenManager>>,
//     ) -> anyhow::Result<Self>
//     // where
//     //         V: DeserializeOwned + Serialize + Clone,
//     //         R: GraphQLResponse,
//     //         Q: GraphQLQuery<R>,
//     //         IQP: FnOnce(&Date, Option<&V>) -> Result<Q, sparko_graphql::Error>,
//     //         CQP: Fn(&Date, &R) -> Option<Result<Q, sparko_graphql::Error>>,
//     //         RI: Fn(R) -> I,
//     //         IN: Fn(&V) -> (DateTime, DateTime),
//     //         I: IntoIterator<Item = V>,
//     //         HP: Fn(&Date) -> String,
//     {
//         Ok(Self{
//             source,
//             hash_key,
//             tz,
//             config: config.clone(),
//             request_manager: request_manager.clone(),
//             // map: HashMap::new(),
//         })
//     }




    // async fn refresh_if_necessary(&self,
    //     open_data_set: OpenMonthlyFixedSlotDataSet<S>,
    //     request_manager: &AuthenticatedRequestManager<OctopusTokenManager>,
    //     source: S,
    // ) -> anyhow::Result<()> {
    //     let data_set = open_data_set.data_set;
    //     let modified = open_data_set.file.metadata()?.modified()?;
    //     let refresh =  data_set.data.len() == 0 || 
    //         SystemTime::now()
    //             .duration_since(modified)
    //             .map(|age| age > self.refresh)
    //             .unwrap_or(false); // If the file is modified in the future, do not refresh it.
        
    //     if refresh  {

    //         let start = &data_set.last_end;
    //         let query = source.initial_query(start)?;
    //         let mut response = request_manager.call(&query).await?;

    //         loop {
    //             let cq = source.continuation_query(&start, &response);

    //             // Self::handle_records(response, &response_iterator, &indexer, &mut data, &mut file)?;

    //             for value in source.items(response) {
    //                 let (start, end, _is_final) = source.index(&value);

    //                 let date: time::Date = start.date();
    //                 if date >= *data_set.start_date && date <= *data_set.next_date {

    //                     let update = if let Some((_attrs, current_value)) = data.get(&index) {
    //                         let v = serde_json::to_string(&value)?;
    //                         let c = serde_json::to_string(current_value)?;
    //                         println!("COMPARE \n{}\n{}\n={}", v, c, v!=c);
    //                         v != c
    //                     }
    //                     else {
    //                         true
    //                     };

    //                     if update {
    //                         data.insert(index, (DataSetAttributes { cached: false, }, value));
    //                         rewrite = true;
    //                     }
    //                 }
    //             }

    //             if let Some(continuation_query) = cq {
    //                 let query = continuation_query?;

    //                 response = request_manager.call(&query).await?;

                    
    //             }
    //             else {
    //                 break;
    //             }
    //         }
            

    //         file.set_times(FileTimes::new().set_modified(SystemTime::now()))?;
    //     };

    //     if rewrite {
    //         if config.verbose 
    //         {
    //             println!("REWRITING FILE {}", path.display());
    //         }
    //         file.seek(SeekFrom::Start(0))?;
    //         file.set_len(0)?;
    //         for (_k, (_a, v)) in &data {
    //             writeln!(file, "{}", serde_json::to_string(&v)?)?;
    //         }
    //     }

    //     Ok(())
    // }
}

/*

impl<V> MonthlyFixedSlotDataSet<V>
where
    V: DeserializeOwned + Serialize,
{
    pub async fn new<R, Q, IQP, CQP, RI, IN, I>(
        hash_key: &str,
        date: &Date,
        slots_per_day: u32,
        timezone: &time_tz::Tz,
        initial_query_provider: IQP,
        continuation_query_provider: CQP,
        response_iterator: RI,
        indexer: IN,
        config: &CacheConfig,
        request_manager: &AuthenticatedRequestManager<OctopusTokenManager>,
    ) -> anyhow::Result<Self>
    where
        R: GraphQLResponse,
        Q: GraphQLQuery<R>,
        IQP: FnOnce(&Date, Option<&V>) -> Result<Q, sparko_graphql::Error>,
        CQP: Fn(&Date, &R) -> Option<Result<Q, sparko_graphql::Error>>,
        RI: Fn(R) -> I,
        IN: Fn(&V) -> DateTime,
        I: IntoIterator<Item = V>,
    {
        let mut dataset = Self::load(hash_key, date, slots_per_day, timezone, config, &indexer)?;
        dataset
            .refresh(
                slots_per_day,
                timezone,
                initial_query_provider,
                continuation_query_provider,
                response_iterator,
                indexer,
                request_manager,
            )
            .await?;
        Ok(dataset)
    }

    pub fn load<VIndex>(
        hash_key: &str,
        date: &Date,
        slots_per_day: u32,
        timezone: &time_tz::Tz,
        config: &CacheConfig,
        indexer: VIndex,
    ) -> anyhow::Result<Self>
    where
        VIndex: Fn(&V) -> DateTime,
    {
        anyhow::ensure!(
            slots_per_day > 0 && 86_400 % slots_per_day == 0,
            "slots_per_day must be a positive divisor of the number of seconds in a day"
        );

        let (path, start_date, end_date) = config.path_for_month(hash_key, date)?;
        let mut dataset = Self {
            start_date,
            end_date,
            data: BTreeMap::new(),
            overflow: BTreeMap::new(),
            path,
            slots_per_day,
            dirty: false,
        };

        if !dataset.path.exists() {
            return Ok(dataset);
        }

        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .open(&dataset.path)?;
        let _guard = file.lock()?;
        for line in BufReader::new(&file).lines() {
            let line = line?;
            match serde_json::from_str::<V>(&line) {
                Ok(value) => {
                    let index = indexer(&value);
                    let date = index.to_timezone(timezone).to_date();
                    if date >= dataset.start_date && date < dataset.end_date {
                        dataset
                            .data
                            .insert(index, (DataSetAttributes { cached: true }, value));
                    }
                }

                Err(error) => {
                    println!("ERROR: failed to read fixed-slot dataset record: {error}");
                    dataset.data.clear();
                    dataset.dirty = true;
                    break;
                }
            }
        }

        Ok(dataset)
    }

    pub async fn refresh<R, Q, IQP, CQP, RI, IN, I>(
        &mut self,
        slots_per_day: u32,
        timezone: &time_tz::Tz,
        initial_query_provider: IQP,
        continuation_query_provider: CQP,
        response_iterator: RI,
        indexer: IN,
        request_manager: &AuthenticatedRequestManager<OctopusTokenManager>,
    ) -> anyhow::Result<()>
    where
        R: GraphQLResponse,
        Q: GraphQLQuery<R>,
        IQP: FnOnce(&Date, Option<&V>) -> Result<Q, sparko_graphql::Error>,
        CQP: Fn(&Date, &R) -> Option<Result<Q, sparko_graphql::Error>>,
        RI: Fn(R) -> I,
        IN: Fn(&V) -> DateTime,
        I: IntoIterator<Item = V>,
    {
        anyhow::ensure!(
            slots_per_day == self.slots_per_day,
            "refresh slots_per_day does not match the loaded dataset"
        );

        if self.is_full(timezone) {
            return Ok(());
        }

        let query = initial_query_provider(&self.start_date, None)?;
        let mut response = request_manager.call(&query).await?;

        loop {
            let next_query = continuation_query_provider(&self.start_date, &response);
            for value in response_iterator(response) {
                let index = indexer(&value);
                let local_date = index.to_timezone(timezone).to_date();
                if local_date >= self.start_date && local_date < self.end_date {
                    self.data
                        .insert(index, (DataSetAttributes { cached: false }, value));
                    self.dirty = true;
                } else {
                    let month_start =
                        Date::from_calendar_date(local_date.year(), local_date.month(), 1)?;
                    self.overflow
                        .entry(month_start)
                        .or_default()
                        .push((index, value));
                }
            }

            if self.is_full(timezone) {
                break;
            }

            if let Some(query) = next_query {
                response = request_manager.call(&query?).await?;
            } else {
                break;
            }
        }

        Ok(())
    }

    pub fn is_full(&self, timezone: &time_tz::Tz) -> bool {
        let slot_seconds = 86_400 / self.slots_per_day as i64;
        let mut date = self.start_date.clone();
        while date < self.end_date {
            let day_start = date.at_midnight(timezone).unix_timestamp();
            let day_end = date.next_day().at_midnight(timezone).unix_timestamp();
            let elapsed = day_end - day_start;
            if elapsed <= 0 || elapsed % slot_seconds != 0 {
                return false;
            }

            let mut timestamp = day_start;
            while timestamp < day_end {
                let Ok(index) = DateTime::from_unix_timestamp(timestamp) else {
                    return false;
                };
                if !self.data.contains_key(&index) {
                    return false;
                }
                timestamp += slot_seconds;
            }
            date = date.next_day();
        }

        true
    }

    pub fn take_overflow(&mut self) -> BTreeMap<Date, Vec<(DateTime, V)>> {
        std::mem::take(&mut self.overflow)
    }

    pub fn add_fetched(&mut self, records: impl IntoIterator<Item = (DateTime, V)>) {
        for (index, value) in records {
            self.data
                .insert(index, (DataSetAttributes { cached: false }, value));
            self.dirty = true;
        }
    }

    pub fn flush(&mut self, config: &CacheConfig) -> anyhow::Result<()> {
        if !self.dirty {
            return Ok(());
        }

        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(&self.path)?;
        let _guard = file.lock()?;
        file.seek(SeekFrom::Start(0))?;
        file.set_len(0)?;
        for (_, value) in self.data.values() {
            writeln!(file, "{}", serde_json::to_string(value)?)?;
        }
        file.set_times(FileTimes::new().set_modified(SystemTime::now()))?;
        if config.verbose {
            println!("FLUSHED FIXED-SLOT DATASET {}", self.path.display());
        }
        self.dirty = false;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;

    #[derive(Deserialize, Serialize)]
    struct Record {
        start_at: DateTime,
    }

    #[test]
    fn completeness_accounts_for_daylight_saving_slot_counts() {
        let timezone = time_tz::timezones::get_by_name("Europe/London").unwrap();
        let start_date = Date::from_calendar_date(2024, time::Month::March, 1).unwrap();
        let end_date = Date::from_calendar_date(2024, time::Month::April, 1).unwrap();
        let mut dataset = MonthlyFixedSlotDataSet {
            start_date: start_date.clone(),
            end_date,
            data: BTreeMap::new(),
            overflow: BTreeMap::new(),
            path: std::path::PathBuf::new(),
            slots_per_day: 48,
            dirty: false,
        };

        let mut date = start_date;
        while date < dataset.end_date {
            let mut timestamp = date.at_midnight(timezone).unix_timestamp();
            let end_timestamp = date.next_day().at_midnight(timezone).unix_timestamp();
            while timestamp < end_timestamp {
                let start_at = DateTime::from_unix_timestamp(timestamp).unwrap();
                dataset.data.insert(
                    start_at.clone(),
                    (DataSetAttributes { cached: true }, Record { start_at }),
                );
                timestamp += 1_800;
            }
            date = date.next_day();
        }

        assert!(dataset.is_full(timezone));
        let missing_slot = DateTime::from_unix_timestamp(
            Date::from_calendar_date(2024, time::Month::March, 31)
                .unwrap()
                .at_midnight(timezone)
                .unix_timestamp(),
        )
        .unwrap();
        dataset.data.remove(&missing_slot);
        assert!(!dataset.is_full(timezone));
    }
}


 */