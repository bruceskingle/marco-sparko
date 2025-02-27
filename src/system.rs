use display_json::DisplayAsJsonPretty;
use serde::{Deserialize, Serialize};

/// Represents a physical site, or address
#[derive(Serialize, Deserialize, Debug, DisplayAsJsonPretty)]
#[serde(rename_all = "camelCase")]
pub struct Site {
    name: String,
}

/// Represents a point of metering, which could be a utility meter (one which is used to bill for usage)
/// or a consumption meter (one which records usage within a Site such as a zwave controller or EV charger)
#[derive(Serialize, Deserialize, Debug, DisplayAsJsonPretty)]
#[serde(rename_all = "camelCase")]
pub struct MeterPoint {
    name: String,
}

pub enum Commodity {
    Electricity,
    Gas,
    Water
}

pub enum MeterType {
    Consumption,
    Export
}