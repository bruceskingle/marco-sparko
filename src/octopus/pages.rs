/*
 * Page components.
 *
 * Each page which needs hooks is a real component so that its hooks are owned by that component
 * rather than by the enclosing Module component, which renders different pages at different times.
 */

mod bills;
mod meter;
mod properties;
mod home_and_ev;

pub use bills::BillsPage;
pub use meter::MeterPage;
pub use properties::PropertiesPage;
pub use home_and_ev::HomeAndEvPage;
