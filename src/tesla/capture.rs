//! Reads the samples a Capture Device took of the gateway, see capture/ in this repo, and saves them as snapshots, so
//! that the app has them for the times it was not running, e.g. while the computer slept.
//!
//! The device keeps what the app's samples use, not whole responses, see capture/src/records.rs, and only since it last
//! started, so it should be read at least as often as it fills up, every 12 hours by default.

use std::sync::Mutex;

use anyhow::anyhow;
use serde::Deserialize;
use time::OffsetDateTime;
use time::format_description::well_known::Rfc3339;

use crate::CacheConfig;

use super::snapshot;

/// What the device says with its records, e.g. when and why it last started.
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct Device {
    boot_time: Option<String>,
    reset_reason: Option<String>,
    records: Option<u64>,
    capacity: Option<u64>,
    free_heap: Option<u64>,
}

#[derive(Deserialize)]
struct Records {
    device: Device,
    records: Vec<serde_json::Value>,
}

pub struct CaptureClient {
    http: reqwest::Client,
    /// The time of the latest record read, to only ask for later ones. Not kept between runs, so the first read reads
    /// everything the device has, saving those not already saved.
    read_until: Mutex<Option<OffsetDateTime>>,
}

impl CaptureClient {
    pub fn new() -> anyhow::Result<CaptureClient> {
        Ok(CaptureClient { http: crate::http::client_builder().build()?, read_until: Mutex::new(None) })
    }

    /// Reads the device's records since the last read, saving those for which there is no snapshot yet, and returns
    /// how many were saved. address is the device's IP address or host name, with a port if not 80.
    pub async fn read(&self, address: &str, cache_config: &CacheConfig, progress: &mut dyn FnMut(String)) -> anyhow::Result<usize> {
        if cache_config.offline {
            return Err(anyhow!("This profile is offline, so nothing can be fetched"));
        }
        let base = if address.contains("://") { address.trim_end_matches('/').to_string() } else { format!("http://{address}") };
        let mut url = reqwest::Url::parse(&format!("{base}/capture/tesla/records"))?;
        let since = *self.read_until.lock().unwrap();
        if let Some(since) = since {
            url.query_pairs_mut().append_pair("since", &since.format(&Rfc3339)?);
        }
        progress(format!("Reading the capture device at {address}"));
        crate::log_info!("Capture device GET {url}");
        let response = self.http.get(url).send().await?;
        let status = response.status();
        let text = response.text().await?;
        if !status.is_success() {
            return Err(anyhow!("the capture device responded {status}: {}", crate::http::summary(&text)));
        }
        let records: Records = serde_json::from_str(&text)?;
        let device = &records.device;
        crate::log_info!("Capture device at {address}: {} records new, {:?} of {:?} kept, started {:?} after {:?}, free heap {:?}",
            records.records.len(), device.records, device.capacity, device.boot_time, device.reset_reason, device.free_heap);

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
        *self.read_until.lock().unwrap() = latest;
        progress(format!("Saved {saved} of the {} new samples from the capture device", records.records.len()));
        Ok(saved)
    }
}
