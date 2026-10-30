//! Polling what Octopus's smart charging (Intelligent Octopus) does, which Octopus only keeps for a few days: the
//! charging it has planned for each device, the charging it has carried out, and the devices' preferences, including
//! whether charging is capped to the hours Octopus guarantees are cheap. Each poll's responses are saved, as they came,
//! in a cache file, see Module::poll.
//!
//! Octopus limits how many requests an account may make, shared with the Octopus app and anything else using the
//! account, and limits it more after it is exceeded, so polling makes at most CALLS_PER_HOUR requests an hour.

use std::collections::VecDeque;
use std::sync::Mutex;
use std::time::{Duration, Instant};

use anyhow::anyhow;
use sparko_graphql::{GraphQLCaller, GraphQLQueryBuilder};
use time::OffsetDateTime;
use time::format_description::well_known::Rfc3339;

use super::context::OctopusContext;
use super::graphql::flex::{completed_dispatches, device_preferences, flex_planned_dispatches, flex_poll};

/// How often the app polls Octopus's smart charging, see polling.rs.
pub const POLL_INTERVAL: Duration = Duration::from_secs(15 * 60);

/// The most requests polling makes in an hour, well within Octopus's limit, so that the Octopus app and others using
/// the account are not affected.
const CALLS_PER_HOUR: usize = 20;

const FLEX_KEY: &str = "flex-";

/// The requests made in the last hour, to keep within CALLS_PER_HOUR.
#[derive(Default)]
pub struct CallBudget {
    calls: Mutex<VecDeque<Instant>>,
}

impl CallBudget {
    /// Counts a request, or fails if CALLS_PER_HOUR have been made in the last hour.
    fn spend(&self) -> anyhow::Result<()> {
        let mut calls = self.calls.lock().unwrap();
        while calls.front().is_some_and(|call| call.elapsed() >= Duration::from_secs(60 * 60)) {
            calls.pop_front();
        }
        if calls.len() >= CALLS_PER_HOUR {
            return Err(anyhow!("Skipped, as polling has made {CALLS_PER_HOUR} requests to Octopus in the last hour"));
        }
        calls.push_back(Instant::now());
        Ok(())
    }
}

/// The charging device whose planned dispatches the combined query, flexPoll, asks for, from an earlier poll's devices,
/// see poll.
#[derive(Default)]
pub struct ChargingDevice(Mutex<Option<String>>);

/// The IDs of the devices which charge, a charge point or vehicle, which are the only ones with planned dispatches, in a
/// response's devices, as JSON.
fn charging_devices(devices: &serde_json::Value) -> Vec<String> {
    devices.as_array().into_iter().flatten()
        .filter(|device| matches!(device["__typename"].as_str(), Some("SmartFlexChargePoint" | "SmartFlexVehicle")))
        .filter_map(|device| device["id"].as_str().map(String::from))
        .collect()
}

/// Polls once for the slot, see Module::poll, saving the responses, and returns a summary, e.g. "charging capped,
/// 2 planned dispatches, 5 completed".
///
/// Once it knows the charging device, from an earlier poll, it is one request, flexPoll, otherwise one for the devices,
/// one for each charging device's planned dispatches, and one for the completed dispatches. Either way the responses
/// are saved as the separate requests' would be, so that what reads them need not know which it was.
pub async fn poll(context: &OctopusContext, slot: Option<OffsetDateTime>, budget: &CallBudget,
    charging_device: &ChargingDevice) -> anyhow::Result<String> {
    if context.cache_config.offline {
        return Err(anyhow!("This profile is offline, so nothing can be fetched"));
    }
    let requested = crate::clock::now_utc();

    let known = charging_device.0.lock().unwrap().clone();
    let (devices, mut planned, completed) = match known {
        Some(device_id) => {
            budget.spend()?;
            let response = context.request_manager.call(&flex_poll::Query::builder()
                .with_account_number(context.account_id.clone())
                .with_device_id(device_id.clone())
                .build()?).await
                // e.g. the device has gone, so that the next poll finds the devices again.
                .inspect_err(|_| *charging_device.0.lock().unwrap() = None)?;
            let mut planned = serde_json::Map::new();
            planned.insert(device_id, serde_json::json!({ "flexPlannedDispatches": response.flex_planned_dispatches_ }));
            (serde_json::to_value(&response.devices_)?, planned, serde_json::to_value(&response.completed_dispatches_)?)
        },
        None => {
            budget.spend()?;
            let preferences = context.request_manager.call(&device_preferences::Query::builder()
                .with_account_number(context.account_id.clone())
                .build()?).await?;
            budget.spend()?;
            let completed = context.request_manager.call(&completed_dispatches::Query::builder()
                .with_account_number(context.account_id.clone())
                .build()?).await?;
            (serde_json::to_value(&preferences.devices_)?, serde_json::Map::new(), serde_json::to_value(&completed.completed_dispatches_)?)
        },
    };

    // Each charging device's planned dispatches, those the combined query did not ask for, e.g. all of them the first
    // time, or a second charging device.
    let charging = charging_devices(&devices);
    let unasked: Vec<&String> = charging.iter().filter(|device_id| !planned.contains_key(*device_id)).collect();
    for device_id in unasked {
        budget.spend()?;
        let response = context.request_manager.call(&flex_planned_dispatches::Query::builder()
            .with_device_id(device_id.clone())
            .build()?).await?;
        planned.insert(device_id.clone(), serde_json::to_value(&response)?);
    }
    *charging_device.0.lock().unwrap() = charging.first().cloned();
    let received = crate::clock::now_utc();

    let saved = serde_json::json!({
        "slot": slot.map(|slot| slot.format(&Rfc3339)).transpose()?,
        "requested": requested.format(&Rfc3339)?,
        "received": received.format(&Rfc3339)?,
        "responses": {
            "devicePreferences": { "devices": devices },
            "flexPlannedDispatches": planned,
            "completedDispatches": { "completedDispatches": completed },
        },
    });
    let path = flex_path(&context.cache_config, slot.unwrap_or(requested))?;
    // Compact, one record a line, as the other cache files are.
    serde_json::to_writer(crate::private_file::create_private_file(&path)?, &saved)?;
    Ok(summary(&saved["responses"]))
}

/// The cache file of the poll of time, e.g. 2026/October/29#flex-172930.
fn flex_path(cache_config: &crate::CacheConfig, time_of: OffsetDateTime) -> anyhow::Result<std::path::PathBuf> {
    let time = time_of.time();
    let key = format!("{FLEX_KEY}{:02}{:02}{:02}", time.hour(), time.minute(), time.second());
    cache_config.path_for_day(&key, &time_of.date().into())
}

/// Saves a poll taken elsewhere, e.g. by a capture device, which is in the same form as poll saves, unless there is one
/// for its time already, returning whether it was saved.
pub fn save_if_new(cache_config: &crate::CacheConfig, saved: &serde_json::Value) -> anyhow::Result<bool> {
    let time = crate::capture_device::record_time(saved).ok_or_else(|| anyhow!("the poll has no time"))?;
    let path = flex_path(cache_config, time)?;
    if path.exists() {
        return Ok(false);
    }
    serde_json::to_writer(crate::private_file::create_private_file(&path)?, saved)?;
    Ok(true)
}

/// The time of the latest poll in the cache, and what it said, see summary.
pub fn latest(cache_config: &crate::CacheConfig) -> Option<(Option<OffsetDateTime>, String)> {
    let saved = cache_config.latest_json(FLEX_KEY)?;
    Some((crate::capture_device::record_time(&saved), summary(&saved["responses"])))
}

/// What a poll's responses say, e.g. "charging capped, 2 planned dispatches, 5 completed".
fn summary(responses: &serde_json::Value) -> String {
    let caps: Vec<&str> = responses["devicePreferences"]["devices"].as_array().into_iter().flatten()
        .filter_map(|device| device["preferences"]["isChargingDurationCapped"].as_str())
        .filter(|cap| *cap != "NOT_APPLICABLE")
        .collect();
    let cap = match caps.as_slice() {
        [] => "no charging cap".to_string(),
        caps if caps.iter().all(|cap| *cap == "ENABLED") => "charging capped".to_string(),
        caps if caps.iter().all(|cap| *cap == "DISABLED") => "charging NOT capped".to_string(),
        caps => format!("charging cap {}", caps.join(", ")),
    };
    let planned: usize = responses["flexPlannedDispatches"].as_object().into_iter().flatten()
        .filter_map(|(_, device)| device["flexPlannedDispatches"].as_array())
        .map(Vec::len)
        .sum();
    let completed = responses["completedDispatches"]["completedDispatches"].as_array().map_or(0, Vec::len);
    format!("{cap}, {planned} planned dispatches, {completed} completed")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_summary_is_read_from_the_saved_responses() {
        let responses = serde_json::json!({
            "devicePreferences": { "devices": [
                { "id": "meter", "__typename": "SmartFlexInverter", "preferences": null },
                { "id": "charger", "__typename": "SmartFlexChargePoint", "preferences": { "isChargingDurationCapped": "ENABLED" } },
            ] },
            "flexPlannedDispatches": { "charger": { "flexPlannedDispatches": [{ "start": "a" }, { "start": "b" }] } },
            "completedDispatches": { "completedDispatches": [{ "start": "c" }] },
        });
        assert_eq!(summary(&responses), "charging capped, 2 planned dispatches, 1 completed");
        assert_eq!(charging_devices(&responses["devicePreferences"]["devices"]), ["charger"]);
    }

    #[test]
    fn the_budget_allows_calls_per_hour_and_no_more() {
        let budget = CallBudget::default();
        for _ in 0..CALLS_PER_HOUR {
            budget.spend().unwrap();
        }
        assert!(budget.spend().is_err());
    }
}
