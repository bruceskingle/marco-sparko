//! Polls the local API of the Tesla Powerwall's gateway, as marco-sparko's src/tesla/local.rs does, which says more.
//!
//! Other programs, e.g. Home Assistant, use the gateway too, so this must never get in their way:
//! - Only GETs of the endpoints in records::ENDPOINTS, which change nothing.
//! - It logs in as the customer and reuses the token it is given, logging in at most once a poll.
//! - A rejected password is removed from the config, so it is not tried again, even after a restart, as repeated
//!   failures lock out the customer login, for every program.

use std::str::FromStr;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use chrono::{DateTime, TimeDelta, Utc};
use croner::Cron;
use embedded_svc::http::client::Client;
use embedded_svc::io::Write;
use esp_idf_svc::http::Method;
use esp_idf_svc::http::client::{Configuration, EspHttpConnection};
use log::{info, warn};
use serde::Deserialize;
use serde_json::json;
use sparko_embedded_std::config::ConfigSpec;
use sparko_embedded_std::config::ConfigSpecValue;
use sparko_embedded_std::config::FeatureConfig;
use sparko_embedded_std::config::TypedValue;
use sparko_embedded_std::feature::FeatureDescriptor;
use sparko_embedded_std::platform::PlatformInitializer;
use sparko_embedded_std::task::scheduler::ScheduledTask;
use sparko_esp_idf::{Esp32Platform, Esp32PlatformInitializer, Feature};

use crate::records::{ENDPOINTS, Record, Response, Store, format_time};
use crate::screen::Screen;

/// The feature's name, which is also its NVS namespace, so at most 15 characters.
const FEATURE_NAME: &str = "TeslaCapture";

/// The name of the password in the config, see forget_password.
const PASSWORD: &str = "tesla_password";

/// Every 5 minutes, on the minute, as the app polls, see marco-sparko's src/polling.rs.
const DEFAULT_SCHEDULE: &str = "0 */5 * * * *";

/// 12 hours of 5 minute samples, which should fit the CYD's RAM, see the free heap in the log after each poll.
const DEFAULT_CAPACITY: i32 = 144;

/// How long to wait before logging in again after the gateway said there had been too many attempts.
const LOCKOUT_WAIT: TimeDelta = TimeDelta::minutes(30);

/// The largest response read, to protect the RAM. meters/aggregates, the largest polled, is about 2.5 KB.
const MAX_BODY: usize = 16 * 1024;

/// The config's names must differ from every other feature's, as the config page's form has one field for each name,
/// and at most 15 characters, as they are NVS keys.
#[derive(FeatureConfig)]
pub struct TeslaConfig {
    /// The gateway's address on the home network.
    #[config(len = 64)]
    tesla_host: String,
    /// The email of the gateway's customer login.
    #[config(len = 64)]
    tesla_email: String,
    /// Optional, so that the config is still valid when a rejected one is removed.
    #[config(len = 64, secret)]
    tesla_password: Option<String>,
    tesla_schedule: Option<Cron>,
    /// How many samples are kept, see DEFAULT_CAPACITY.
    tesla_capacity: Option<i32>,
}

pub struct TeslaCapture {}

impl TeslaCapture {
    pub fn new() -> Self {
        Self {}
    }
}

impl Feature for TeslaCapture {
    type Config = TeslaConfig;

    fn init(&self, _initializer: &mut Esp32PlatformInitializer) -> anyhow::Result<FeatureDescriptor> {
        Ok(FeatureDescriptor {
            name: FEATURE_NAME.to_string(),
            config: TeslaConfig::to_config_spec()?,
        })
    }

    fn start(
        &mut self,
        sparko: &mut Esp32Platform,
        initializer: &mut Esp32PlatformInitializer,
        config: TeslaConfig,
    ) -> anyhow::Result<()> {
        let schedule_spec = config
            .tesla_schedule
            .map(|schedule| schedule.pattern.to_string())
            .unwrap_or_else(|| DEFAULT_SCHEDULE.to_string());
        let capacity = config.tesla_capacity.filter(|capacity| *capacity > 0).unwrap_or(DEFAULT_CAPACITY);
        let store = Arc::new(Mutex::new(Store::new(capacity as usize)));
        let device = Device::new();

        let mut poll = PollTask {
            gateway: Gateway {
                host: config.tesla_host,
                email: config.tesla_email,
                password: config.tesla_password,
                token: None,
                no_login_until: None,
            },
            store: store.clone(),
            schedule: Cron::from_str(&schedule_spec)?,
            next_slot: Utc::now(),
        };
        poll.next_slot = poll.schedule.find_next_occurrence(&Utc::now(), false)?;

        initializer.add_task(Box::new(Screen::new(&sparko.display_manager, store.clone())?), "*/2 * * * * *")?;

        // A first poll now, as the app does, so there is something to show without waiting for the first slot.
        poll.poll(sparko, None);
        initializer.add_task(Box::new(poll), &schedule_spec)?;

        sparko.server_manager.on("/capture/tesla/records", Method::Get, move |request| {
            let since = since(request.uri());
            let mut response = request.into_response(200, None, &[("Content-Type", "application/json")])?;
            let store = store.lock().unwrap();
            // Written a record at a time, as all of them would not fit in the RAM as one piece of JSON.
            response.write_all(b"{\"device\":")?;
            response.write_all(&serde_json::to_vec(&device.to_json(&store))?)?;
            response.write_all(b",\"records\":[")?;
            for (index, record) in store.since(since).enumerate() {
                if index > 0 {
                    response.write_all(b",")?;
                }
                response.write_all(&serde_json::to_vec(&record.to_json(CAPTURED_BY))?)?;
            }
            response.write_all(b"]}")?;
            Ok(())
        })?;

        Ok(())
    }
}

/// The since parameter of the query string, an RFC 3339 time, e.g. ?since=2026-10-29T07:30:00Z
fn since(uri: &str) -> Option<DateTime<Utc>> {
    let (_, query) = uri.split_once('?')?;
    url::form_urlencoded::parse(query.as_bytes())
        .find(|(name, _)| name == "since")
        .and_then(|(_, value)| DateTime::parse_from_rfc3339(&value).ok())
        .map(|time| time.with_timezone(&Utc))
}

/// What took the records, in each one's capturedBy.
const CAPTURED_BY: &str = concat!(env!("CARGO_PKG_NAME"), " ", env!("CARGO_PKG_VERSION"));

/// What the app is told about the device with the records.
struct Device {
    /// When the device last started, and why, e.g. Brownout, which shows power dips.
    boot_time: DateTime<Utc>,
    reset_reason: &'static str,
}

impl Device {
    fn new() -> Device {
        let up_time = TimeDelta::microseconds(unsafe { esp_idf_sys::esp_timer_get_time() });
        Device {
            boot_time: Utc::now() - up_time,
            reset_reason: reset_reason(),
        }
    }

    fn to_json(&self, store: &Store) -> serde_json::Value {
        json!({
            "name": env!("CARGO_PKG_NAME"),
            "version": env!("CARGO_PKG_VERSION"),
            "now": format_time(Utc::now()),
            "bootTime": format_time(self.boot_time),
            "resetReason": self.reset_reason,
            "records": store.len(),
            "capacity": store.capacity(),
            "freeHeap": unsafe { esp_idf_sys::esp_get_free_heap_size() },
            "minimumFreeHeap": unsafe { esp_idf_sys::esp_get_minimum_free_heap_size() },
        })
    }
}

fn reset_reason() -> &'static str {
    #[allow(non_upper_case_globals)]
    match unsafe { esp_idf_sys::esp_reset_reason() } {
        esp_idf_sys::esp_reset_reason_t_ESP_RST_POWERON => "PowerOn",
        esp_idf_sys::esp_reset_reason_t_ESP_RST_EXT => "ExternalPin",
        esp_idf_sys::esp_reset_reason_t_ESP_RST_SW => "Software",
        esp_idf_sys::esp_reset_reason_t_ESP_RST_PANIC => "Panic",
        esp_idf_sys::esp_reset_reason_t_ESP_RST_INT_WDT => "InterruptWatchdog",
        esp_idf_sys::esp_reset_reason_t_ESP_RST_TASK_WDT => "TaskWatchdog",
        esp_idf_sys::esp_reset_reason_t_ESP_RST_WDT => "OtherWatchdog",
        esp_idf_sys::esp_reset_reason_t_ESP_RST_DEEPSLEEP => "DeepSleep",
        esp_idf_sys::esp_reset_reason_t_ESP_RST_BROWNOUT => "Brownout",
        esp_idf_sys::esp_reset_reason_t_ESP_RST_SDIO => "Sdio",
        _ => "Unknown",
    }
}

/// The gateway did not accept the email and password, which must not be tried again until they are changed.
#[derive(Debug)]
struct PasswordRejected;

impl std::fmt::Display for PasswordRejected {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "the gateway did not accept the email and password, which have been removed, set them again")
    }
}

impl std::error::Error for PasswordRejected {}

/// The gateway responded with a status other than success, after which the poll goes on to the next endpoint, unlike
/// a failure to reach it, which stops the poll.
#[derive(Debug)]
struct StatusError(String);

impl std::fmt::Display for StatusError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for StatusError {}

struct Gateway {
    host: String,
    email: String,
    password: Option<String>,
    token: Option<String>,
    /// Set when the gateway says there have been too many login attempts.
    no_login_until: Option<DateTime<Utc>>,
}

impl Gateway {
    fn url(&self, endpoint: &str) -> String {
        format!("https://{}/api/{endpoint}", self.host)
    }

    fn login(&mut self, client: &mut Client<EspHttpConnection>) -> anyhow::Result<()> {
        let Some(password) = &self.password else {
            anyhow::bail!("there is no Tesla password, set it on the config page");
        };
        if let Some(until) = self.no_login_until.filter(|until| *until > Utc::now()) {
            anyhow::bail!("not logging in until {}, after too many attempts", format_time(until));
        }
        info!("Tesla gateway POST login/Basic");
        // force_sm_off is always false, as true stops the system on some versions of the gateway's software.
        let body = serde_json::to_vec(&json!({
            "username": "customer",
            "email": self.email,
            "password": password,
            "force_sm_off": false,
        }))?;
        let length = body.len().to_string();
        let headers = [("Content-Type", "application/json"), ("Content-Length", length.as_str())];
        let url = self.url("login/Basic");
        let mut request = client.post(&url, &headers)?;
        request.write_all(&body)?;
        let mut response = request.submit()?;
        let status = response.status();
        let body = read_body(&mut response)?;
        match status {
            401 | 403 => return Err(PasswordRejected.into()),
            429 => {
                self.no_login_until = Some(Utc::now() + LOCKOUT_WAIT);
                anyhow::bail!("the gateway has locked out the customer login after too many attempts");
            }
            status if !(200..300).contains(&status) => {
                anyhow::bail!("the gateway responded {status} to login: {}", summary(&body))
            }
            _ => {}
        }
        #[derive(Deserialize)]
        struct LoginResponse {
            token: String,
        }
        self.token = Some(serde_json::from_slice::<LoginResponse>(&body)?.token);
        Ok(())
    }

    fn send_get(&self, client: &mut Client<EspHttpConnection>, endpoint: &str) -> anyhow::Result<(u16, Vec<u8>)> {
        info!("Tesla gateway GET {endpoint}");
        let authorization = format!("Bearer {}", self.token.as_deref().unwrap_or_default());
        let headers = [("Authorization", authorization.as_str())];
        let url = self.url(endpoint);
        let mut response = client.request(Method::Get, &url, &headers)?.submit()?;
        let status = response.status();
        Ok((status, read_body(&mut response)?))
    }

    /// GETs endpoint, logging in again if the token is rejected, unless this poll has already logged in, see poll.
    fn get(&mut self, client: &mut Client<EspHttpConnection>, endpoint: &str, logged_in: &mut bool) -> anyhow::Result<Vec<u8>> {
        let (mut status, mut body) = self.send_get(client, endpoint)?;
        if matches!(status, 401 | 403) && !*logged_in {
            info!("The Tesla gateway did not accept the token, logging in again");
            self.token = None;
            *logged_in = true;
            self.login(client)?;
            (status, body) = self.send_get(client, endpoint)?;
        }
        if !(200..300).contains(&status) {
            return Err(StatusError(format!("the gateway responded {status} to {endpoint}: {}", summary(&body))).into());
        }
        Ok(body)
    }
}

fn read_body(response: &mut embedded_svc::http::client::Response<&mut EspHttpConnection>) -> anyhow::Result<Vec<u8>> {
    let mut body = Vec::new();
    let mut buffer = [0u8; 512];
    loop {
        let read = response.read(&mut buffer)?;
        if read == 0 {
            return Ok(body);
        }
        if body.len() + read > MAX_BODY {
            anyhow::bail!("the response is longer than {MAX_BODY} bytes");
        }
        body.extend_from_slice(&buffer[..read]);
    }
}

/// The start of a response, for an error message.
fn summary(body: &[u8]) -> String {
    String::from_utf8_lossy(&body[..body.len().min(100)]).into_owned()
}

struct PollTask {
    gateway: Gateway,
    store: Arc<Mutex<Store>>,
    /// The schedule this task was added with, to know which slot each run is for, see slot.
    schedule: Cron,
    next_slot: DateTime<Utc>,
}

impl ScheduledTask<Esp32Platform> for PollTask {
    fn run(&mut self, sparko: &mut Esp32Platform) -> anyhow::Result<()> {
        let slot = self.slot()?;
        self.poll(sparko, slot);
        Ok(())
    }

    fn name(&self) -> &str {
        "Tesla Capture"
    }
}

impl PollTask {
    /// The slot this run is for, the latest time of the schedule which has passed, as the scheduler runs a task at
    /// or a little after its time, later if another task held it up.
    fn slot(&mut self) -> anyhow::Result<Option<DateTime<Utc>>> {
        let now = Utc::now();
        let mut slot = None;
        while self.next_slot <= now {
            slot = Some(self.next_slot);
            self.next_slot = self.schedule.find_next_occurrence(&self.next_slot, false)?;
        }
        Ok(slot)
    }

    /// Polls the gateway, saving the record, or the error if the gateway could not be reached, for the screen.
    fn poll(&mut self, sparko: &mut Esp32Platform, slot: Option<DateTime<Utc>>) {
        let requested = Utc::now();
        let result = self.fetch(slot, requested);
        let heap = unsafe { esp_idf_sys::esp_get_free_heap_size() };
        let minimum_heap = unsafe { esp_idf_sys::esp_get_minimum_free_heap_size() };
        let mut store = self.store.lock().unwrap();
        match result {
            Ok(record) => {
                store.add(record);
                info!("Tesla poll saved, {} records, free heap {heap}, minimum {minimum_heap}", store.len());
            }
            Err(error) => {
                if error.is::<PasswordRejected>() {
                    self.gateway.password = None;
                    if let Err(error) = forget_password(sparko) {
                        log::error!("Unable to remove the rejected Tesla password from the config, {error:#}");
                    }
                }
                warn!("Tesla poll failed, {error:#}");
                store.failed(requested, format!("{error:#}"));
            }
        }
    }

    /// Requests each of the ENDPOINTS in turn, on one connection, logging in first if there is no token.
    fn fetch(&mut self, slot: Option<DateTime<Utc>>, requested: DateTime<Utc>) -> anyhow::Result<Record> {
        let mut client = Client::wrap(EspHttpConnection::new(&Configuration {
            timeout: Some(Duration::from_secs(10)),
            ..Default::default()
        })?);
        let mut logged_in = false;
        if self.gateway.token.is_none() {
            logged_in = true;
            self.gateway.login(&mut client)?;
        }
        let mut responses = Vec::with_capacity(ENDPOINTS.len());
        for endpoint in ENDPOINTS {
            let response = self
                .gateway
                .get(&mut client, endpoint, &mut logged_in)
                .and_then(|body| Response::parse(endpoint, &body));
            match response {
                Ok(response) => responses.push(Ok(response)),
                // Recorded as the app does, see marco-sparko's snapshot::fetch, but anything else, e.g. the gateway
                // not answering, stops the poll, rather than waiting for it to time out on every endpoint.
                Err(error) if error.is::<StatusError>() || error.is::<serde_json::Error>() => {
                    warn!("Unable to fetch {endpoint} from the Tesla gateway, {error:#}");
                    responses.push(Err(format!("{error:#}").into()));
                }
                Err(error) => return Err(error),
            }
        }
        Ok(Record { slot, requested, received: Utc::now(), responses })
    }
}

/// Removes the password from the config, so that it is not tried again after a restart.
fn forget_password(sparko: &Esp32Platform) -> anyhow::Result<()> {
    let holder = sparko
        .config_manager
        .features
        .get(FEATURE_NAME)
        .ok_or_else(|| anyhow::anyhow!("there is no config for {FEATURE_NAME}"))?;
    let mut config = holder.inner.lock().unwrap();
    if let Some(value) = config.spec.map.get_mut(PASSWORD) {
        holder.config_store.remove(PASSWORD, value)?;
    }
    Ok(())
}
