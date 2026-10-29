//! Reads the samples the profile's capture devices took of the gateway, see capture/ in this repo and
//! capture_device.rs, and saves them as snapshots, so that the app has them for the times it was not running, e.g.
//! while the computer slept.
//!
//! A device keeps what the app's samples use, not whole responses, see capture/src/records.rs, on its SD card if it
//! has one, otherwise only since it last started, in RAM.

use std::collections::HashMap;
use std::sync::Mutex;

use serde::Deserialize;
use time::OffsetDateTime;
use time::format_description::well_known::Rfc3339;

use crate::CacheConfig;
use crate::capture_device::{DeviceClient, DeviceProfile, StatusError};

use super::snapshot;

/// What the device says with its records, e.g. when and why it last started.
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct Device {
    boot_time: Option<String>,
    reset_reason: Option<String>,
    storage: Option<String>,
    free_heap: Option<u64>,
}

#[derive(Deserialize)]
struct Records {
    device: Device,
    records: Vec<serde_json::Value>,
}

pub struct CaptureClient {
    /// The time of the latest record read from each device, by its ID, to only ask for later ones. Not kept between
    /// runs: the first read of a device asks for those after the latest snapshot in the cache.
    read_until: Mutex<HashMap<String, OffsetDateTime>>,
}

impl CaptureClient {
    pub fn new() -> CaptureClient {
        CaptureClient { read_until: Mutex::new(HashMap::new()) }
    }

    /// Reads the records of each device which captures the gateway, saving those for which there is no snapshot yet.
    /// Returns false if none of them does, e.g. there are none, so that the app polls the gateway itself, and an error
    /// if none which does could be read.
    pub async fn read(&self, devices: &[DeviceProfile], cache_config: &CacheConfig, progress: &mut dyn FnMut(String)) -> anyhow::Result<bool> {
        let mut captures = false;
        let mut first_error = None;
        let mut read_any = false;
        for device in devices {
            match self.read_device(device, cache_config, progress).await {
                Ok(()) => {
                    captures = true;
                    read_any = true;
                }
                // It does not capture the gateway.
                Err(error) if error.downcast_ref::<StatusError>().is_some_and(|error| error.status == 404) => {}
                Err(error) => {
                    captures = true;
                    crate::log_warning!("Unable to read the capture device {}, {error:#}", device.id);
                    first_error.get_or_insert(error);
                }
            }
        }
        match first_error {
            Some(error) if !read_any => Err(error),
            _ => Ok(captures),
        }
    }

    async fn read_device(&self, device: &DeviceProfile, cache_config: &CacheConfig, progress: &mut dyn FnMut(String)) -> anyhow::Result<()> {
        if cache_config.offline {
            return Err(anyhow::anyhow!("This profile is offline, so nothing can be fetched"));
        }
        let known = self.read_until.lock().unwrap().get(&device.id).copied();
        let since = known.or_else(|| snapshot::latest(cache_config).as_ref().and_then(snapshot::time_of));
        let query: Vec<(&str, String)> = since.map(|since| since.format(&Rfc3339)).transpose()?
            .into_iter().map(|since| ("since", since)).collect();
        progress(format!("Reading the capture device {}", device.id));
        crate::log_info!("Capture device {} GET /api/capture/tesla/records {query:?}", device.id);
        let json = DeviceClient::for_device(device)?.get_query("capture/tesla/records", &query).await?;
        let records: Records = serde_json::from_value(json)?;
        let info = &records.device;
        crate::log_info!("Capture device {}: {} records new, kept on {:?}, started {:?} after {:?}, free heap {:?}",
            device.id, records.records.len(), info.storage, info.boot_time, info.reset_reason, info.free_heap);

        let mut saved = 0;
        let mut latest = since;
        for record in &records.records {
            let Some(time) = snapshot::time_of(record) else {
                crate::log_warning!("A capture device record has no time, {}", crate::http::summary(&record.to_string()));
                continue;
            };
            if snapshot::save_if_new(cache_config, record)? {
                saved += 1;
            }
            latest = latest.max(Some(time));
        }
        if let Some(latest) = latest {
            self.read_until.lock().unwrap().insert(device.id.clone(), latest);
        }
        progress(format!("Saved {saved} of the {} new samples from the capture device {}", records.records.len(), device.id));
        Ok(())
    }
}
