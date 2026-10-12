mod list;
mod monthly;
mod multi_query;
mod ordered_list;
mod single_record;

pub use list::ListDataSet;
pub use monthly::MonthlyDataSet;
pub use multi_query::MultiQueryDataSet;
pub use ordered_list::OrderedListDataSet;
pub use single_record::SingleRecordDataSet;

pub struct DataSetAttributes {
    pub cached: bool,
}
