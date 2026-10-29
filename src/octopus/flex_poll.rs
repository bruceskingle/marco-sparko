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
use super::graphql::flex::{completed_dispatches, device_preferences, flex_planned_dispatches};

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

/// The name of an enum value, as it is in the response, e.g. ENABLED.
fn enum_name<T: serde::Serialize>(value: &T) -> String {
    serde_json::to_value(value).ok().and_then(|json| json.as_str().map(String::from)).unwrap_or_default()
}

/// Polls once for the slot, see Module::poll, saving the responses, and returns a summary, e.g. "charging capped,
/// 2 planned dispatches, 5 completed".
pub async fn poll(context: &OctopusContext, slot: Option<OffsetDateTime>, budget: &CallBudget) -> anyhow::Result<String> {
    if context.cache_config.offline {
        return Err(anyhow!("This profile is offline, so nothing can be fetched"));
    }
    let requested = crate::clock::now_utc();

    budget.spend()?;
    let preferences = context.request_manager.call(&device_preferences::Query::builder()
        .with_account_number(context.account_id.clone())
        .build()?).await?;

    // Only devices which charge have planned dispatches.
    let charging_devices: Vec<String> = preferences.devices_.iter()
        .filter(|device| matches!(device,
            device_preferences::SmartFlexDeviceInterface::SmartFlexChargePoint(_)
            | device_preferences::SmartFlexDeviceInterface::SmartFlexVehicle(_)))
        .map(|device| device.as_smart_flex_device_interface().id_.clone())
        .collect();
    let mut planned = serde_json::Map::new();
    let mut planned_count = 0;
    for device_id in &charging_devices {
        budget.spend()?;
        let response = context.request_manager.call(&flex_planned_dispatches::Query::builder()
            .with_device_id(device_id.clone())
            .build()?).await?;
        planned_count += response.flex_planned_dispatches_.len();
        planned.insert(device_id.clone(), serde_json::to_value(&response)?);
    }

    budget.spend()?;
    let completed = context.request_manager.call(&completed_dispatches::Query::builder()
        .with_account_number(context.account_id.clone())
        .build()?).await?;
    let received = crate::clock::now_utc();

    let saved = serde_json::json!({
        "slot": slot.map(|slot| slot.format(&Rfc3339)).transpose()?,
        "requested": requested.format(&Rfc3339)?,
        "received": received.format(&Rfc3339)?,
        "responses": {
            "devicePreferences": serde_json::to_value(&preferences)?,
            "flexPlannedDispatches": planned,
            "completedDispatches": serde_json::to_value(&completed)?,
        },
    });
    let time_of = slot.unwrap_or(requested);
    let time = time_of.time();
    let key = format!("{FLEX_KEY}{:02}{:02}{:02}", time.hour(), time.minute(), time.second());
    let path = context.cache_config.path_for_day(&key, &time_of.date().into())?;
    // Compact, one record a line, as the other cache files are.
    serde_json::to_writer(crate::private_file::create_private_file(&path)?, &saved)?;

    let caps: Vec<String> = preferences.devices_.iter()
        .filter_map(|device| device.as_smart_flex_device_interface().preferences_.as_ref())
        .map(|preferences| enum_name(&preferences.as_smart_flex_device_preferences_interface().is_charging_duration_capped_))
        .filter(|cap| cap != "NOT_APPLICABLE")
        .collect();
    let cap = match caps.as_slice() {
        [] => "no charging cap".to_string(),
        caps if caps.iter().all(|cap| cap == "ENABLED") => "charging capped".to_string(),
        caps if caps.iter().all(|cap| cap == "DISABLED") => "charging NOT capped".to_string(),
        caps => format!("charging cap {}", caps.join(", ")),
    };
    Ok(format!("{cap}, {planned_count} planned dispatches, {} completed", completed.completed_dispatches_.len()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_budget_allows_calls_per_hour_and_no_more() {
        let budget = CallBudget::default();
        for _ in 0..CALLS_PER_HOUR {
            budget.spend().unwrap();
        }
        assert!(budget.spend().is_err());
    }
}
