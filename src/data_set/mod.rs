mod cancel;
mod list;
mod monthly;
mod monthly_fixed_slot;
mod multi_query;
mod ordered_list;
mod single_record;

pub use cancel::CancelToken;
pub use list::ListDataSet;
pub use monthly::MonthlyDataSet;
pub use monthly_fixed_slot::*;
pub use multi_query::MultiQueryDataSet;
pub use ordered_list::OrderedListDataSet;
pub use single_record::SingleRecordDataSet;

/// DataSets represent the caching strategy for the application. Data is obtained from an API in the form of JSON serializable objects.
/// A DataSet is a collection of one or more of these objects, there are separate DataSet implementations for various styles of these 
/// collections such as
/// * A single object
/// * A list of objects
/// * A set of samples each of which has a start and end DateTime where the end of one sample equals the start of the next, these samples
/// are cached in sets for a month of data.
/// Each DataSet is written to a single file on the local file system, that file is the master copy and the synchronization control
/// between processes.
/// When a caller requests a DataSet then the local file for that DataSet is created if it does not exist and then locked.
/// The DataSet is read from the file. If the set is empty or the modify date is older than some value
/// chosen by the caller, then the API is called to see if there is more data, the file remains locked for this period. If more data
/// is received then the file is updated and then unlocked. The DataSet is then returned to the caller as an owned object. There is no
/// centralized in memory cache of the DataSet if another caller requests it then the file is re-read (since it could have been updated
/// by another process in the mean time).
/// 
/// A DataSetManager is a class (struct with associated functions) which returns one or more DataSets. In the case of the monthly sampled
/// DataSet the API may return values outside the date range for the requested DataSet and that data may be cached and written to local
/// files for efficiency. The manager is stateless and does not cache DataSets between calls.
/// 
/// An OpenDataSet is a DataSet with an open file handle to the locked local file. It is used only by the manager for the duration of a
/// request.
/// 
/// Values in some DataSets may be final or not, a non-final value is subject to later update on the system providing the API, a final
/// value is not, so there is never any point refetching a final value, but if a refresh of the cached file is required then all
/// non-final values will be fetched again and if they have changed then will be updated in the cache file.

#[derive(Clone, Copy)]
pub struct DataSetAttributes {
    pub cached: bool,
}
