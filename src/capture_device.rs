//! Capture devices: ESP32s, see capture/ in this repo, which poll what the modules need while the computer sleeps, and
//! which the app manages through their HTTPS API, see sparko-embedded-std's api.rs.
//!
//! A device's certificate is one it made itself, so the app pins it: it notes the certificate's fingerprint the first
//! time it connects, checks it against what the device says in /api/info, and from then on only talks to a device with
//! that certificate, see PinnedCertificate. Adopting a device, see adopt, gives it a key the app made, which every
//! request but /api/info then needs.

use std::sync::{Arc, Mutex};
use std::time::Duration;

use anyhow::anyhow;
use rustls::client::danger::{HandshakeSignatureValid, ServerCertVerified, ServerCertVerifier};
use rustls::crypto::CryptoProvider;
use rustls::pki_types::{CertificateDer, ServerName, UnixTime};
use rustls::{DigitallySignedStruct, SignatureScheme};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};

/// A device the profile has adopted, in its settings, see profile::update_capture_devices.
#[derive(Clone, Serialize, Deserialize, Default, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct DeviceProfile {
    /// The device's ID, its Wi-Fi MAC address, which never changes.
    pub id: String,
    /// Its IP address or host name, which may change.
    pub address: String,
    /// The SHA-256 of its certificate, as lower case hex, see PinnedCertificate.
    pub fingerprint: String,
    /// The key the app gave it when it adopted it, 32 bytes as hex.
    pub key: String,
}

/// Debug output is used in verbose mode, so the key is hidden.
impl std::fmt::Debug for DeviceProfile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DeviceProfile")
            .field("id", &self.id)
            .field("address", &self.address)
            .field("fingerprint", &self.fingerprint)
            .field("key", &sparko_graphql::redact::REDACTED)
            .finish()
    }
}

/// What a device says it is, from /api/info.
#[derive(Clone, Deserialize, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct DeviceInfo {
    pub id: String,
    pub board: String,
    pub firmware: Software,
    pub platform: Software,
    pub certificate_fingerprint: String,
    pub adopted: bool,
    pub up_time_seconds: u64,
    pub features: Vec<FeatureState>,
}

#[derive(Clone, Deserialize, Debug, PartialEq)]
pub struct Software {
    pub name: String,
    pub version: String,
}

#[derive(Clone, Deserialize, Debug, PartialEq)]
pub struct FeatureState {
    pub name: String,
    pub enabled: bool,
}

/// The device responded with a status other than success, with the reason it gave, if any, e.g. 404 for a feature it
/// does not have.
#[derive(Debug)]
pub struct StatusError {
    pub status: u16,
    pub message: String,
}

impl std::fmt::Display for StatusError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} ({})", self.message, self.status)
    }
}

impl std::error::Error for StatusError {}

fn sha256_hex(bytes: &[u8]) -> String {
    Sha256::digest(bytes).iter().map(|byte| format!("{byte:02x}")).collect()
}

/// Accepts only the certificate with the expected fingerprint, or, the first time, when there is none yet, any, noting
/// its fingerprint in seen, to check against what the device says and pin from then on.
#[derive(Debug)]
struct PinnedCertificate {
    expected: Option<String>,
    seen: Arc<Mutex<Option<String>>>,
    provider: Arc<CryptoProvider>,
}

impl ServerCertVerifier for PinnedCertificate {
    fn verify_server_cert(&self, end_entity: &CertificateDer<'_>, _intermediates: &[CertificateDer<'_>],
        _server_name: &ServerName<'_>, _ocsp_response: &[u8], _now: UnixTime) -> Result<ServerCertVerified, rustls::Error> {
        let fingerprint = sha256_hex(end_entity.as_ref());
        *self.seen.lock().unwrap() = Some(fingerprint.clone());
        match &self.expected {
            Some(expected) if *expected != fingerprint => Err(rustls::Error::General(format!(
                "the device's certificate is not the one it had when it was adopted ({fingerprint} instead of {expected}), \
                 it may have been reset, or something else may be at its address"))),
            _ => Ok(ServerCertVerified::assertion()),
        }
    }

    // The certificate's signature still proves the device has its key.
    fn verify_tls12_signature(&self, message: &[u8], cert: &CertificateDer<'_>, dss: &DigitallySignedStruct)
        -> Result<HandshakeSignatureValid, rustls::Error> {
        rustls::crypto::verify_tls12_signature(message, cert, dss, &self.provider.signature_verification_algorithms)
    }

    fn verify_tls13_signature(&self, message: &[u8], cert: &CertificateDer<'_>, dss: &DigitallySignedStruct)
        -> Result<HandshakeSignatureValid, rustls::Error> {
        rustls::crypto::verify_tls13_signature(message, cert, dss, &self.provider.signature_verification_algorithms)
    }

    fn supported_verify_schemes(&self) -> Vec<SignatureScheme> {
        self.provider.signature_verification_algorithms.supported_schemes()
    }
}

/// A connection to one device's API.
pub struct DeviceClient {
    http: reqwest::Client,
    base: String,
    key: Option<String>,
    seen: Arc<Mutex<Option<String>>>,
}

impl DeviceClient {
    /// A client for the device at address, which only accepts the certificate with fingerprint, if given, and sends
    /// key, if given.
    pub fn new(address: &str, fingerprint: Option<&str>, key: Option<&str>) -> anyhow::Result<DeviceClient> {
        let provider = Arc::new(rustls::crypto::ring::default_provider());
        let seen = Arc::new(Mutex::new(None));
        let verifier = PinnedCertificate { expected: fingerprint.map(String::from), seen: seen.clone(), provider: provider.clone() };
        let tls = rustls::ClientConfig::builder_with_provider(provider)
            .with_safe_default_protocol_versions()?
            .dangerous()
            .with_custom_certificate_verifier(Arc::new(verifier))
            .with_no_client_auth();
        let http = crate::http::client_builder()
            .use_preconfigured_tls(tls)
            .connect_timeout(Duration::from_secs(5))
            .build()?;
        Ok(DeviceClient { http, base: format!("https://{}/api", address.trim()), key: key.map(String::from), seen })
    }

    /// For a device the profile has adopted.
    pub fn for_device(device: &DeviceProfile) -> anyhow::Result<DeviceClient> {
        DeviceClient::new(&device.address, Some(&device.fingerprint), Some(&device.key))
    }

    /// The fingerprint of the certificate the device presented, once connected.
    pub fn seen_fingerprint(&self) -> Option<String> {
        self.seen.lock().unwrap().clone()
    }

    async fn send(&self, request: reqwest::RequestBuilder) -> anyhow::Result<Value> {
        let request = match &self.key {
            Some(key) => request.bearer_auth(key),
            None => request,
        };
        let response = request.send().await?;
        let status = response.status();
        let text = response.text().await?;
        if !status.is_success() {
            // The API's errors are JSON, but not the HTTP server's own, e.g. 404 for a path no feature has.
            let message = serde_json::from_str::<Value>(&text).ok()
                .and_then(|json| json.get("error").and_then(|error| error.as_str()).map(String::from))
                .unwrap_or_else(|| crate::http::summary(&text));
            return Err(StatusError { status: status.as_u16(), message }.into());
        }
        serde_json::from_str(&text).map_err(|_| anyhow!("the device responded {status}: {}", crate::http::summary(&text)))
    }

    pub async fn get(&self, path: &str) -> anyhow::Result<Value> {
        self.send(self.http.get(format!("{}/{path}", self.base))).await
    }

    pub async fn get_query(&self, path: &str, query: &[(&str, String)]) -> anyhow::Result<Value> {
        self.send(self.http.get(format!("{}/{path}", self.base)).query(query)).await
    }

    pub async fn post(&self, path: &str, body: &Value) -> anyhow::Result<Value> {
        self.send(self.http.post(format!("{}/{path}", self.base)).json(body)).await
    }

    pub async fn info(&self) -> anyhow::Result<DeviceInfo> {
        Ok(serde_json::from_value(self.get("info").await?)?)
    }

    /// Asks the device to show a pairing code, returning how long it lasts.
    pub async fn start_pairing(&self) -> anyhow::Result<Duration> {
        let json = self.post("adopt/start", &serde_json::json!({})).await?;
        Ok(Duration::from_secs(json["expiresInSeconds"].as_u64().unwrap_or_default()))
    }

    pub async fn release(&self) -> anyhow::Result<()> {
        self.post("release", &serde_json::json!({})).await.map(|_| ())
    }

    pub async fn restart(&self) -> anyhow::Result<()> {
        self.post("restart", &serde_json::json!({})).await.map(|_| ())
    }

    /// Every feature's config, see sparko-embedded-std's config_json.rs.
    pub async fn config(&self) -> anyhow::Result<Value> {
        self.get("config").await
    }

    /// Changes the config, which takes effect when the device restarts, returning the new config.
    pub async fn update_config(&self, update: &Value) -> anyhow::Result<Value> {
        let json = self.post("config", update).await?;
        Ok(json["config"].clone())
    }
}

/// The devices the profile has adopted, as saved now, not when the app started, as the Devices page may have adopted
/// or released one since.
pub fn adopted_devices(profile_name: &str) -> Vec<DeviceProfile> {
    crate::profile::fetch_active_profile(&Some(profile_name.to_string()))
        .map(|profile| profile.active_profile.settings.capture_devices)
        .unwrap_or_default()
}

/// A device found at an address, not yet adopted, with what it says it is and the certificate it presented.
#[derive(Clone, Debug, PartialEq)]
pub struct FoundDevice {
    pub address: String,
    pub info: DeviceInfo,
    pub fingerprint: String,
}

/// Asks the device at address what it is, checking that the certificate it presented is the one it says it has, so
/// that it can be pinned.
pub async fn find(address: &str) -> anyhow::Result<FoundDevice> {
    let address = address.trim();
    let client = DeviceClient::new(address, None, None)?;
    let info = client.info().await.map_err(|error| anyhow!("no capture device answered at {address}, {error:#}"))?;
    let fingerprint = client.seen_fingerprint().ok_or_else(|| anyhow!("the device presented no certificate"))?;
    if fingerprint != info.certificate_fingerprint {
        return Err(anyhow!("the certificate presented at {address} is not the one the device there says it has, \
            something else may be in between"));
    }
    Ok(FoundDevice { address: address.to_string(), info, fingerprint })
}

/// Asks a found device to show its pairing code, see adopt.
pub async fn start_pairing(found: &FoundDevice) -> anyhow::Result<Duration> {
    DeviceClient::new(&found.address, Some(&found.fingerprint), None)?.start_pairing().await
}

/// Adopts a found device with the code on its screen, giving it a new key, and returns it for the profile.
pub async fn adopt(found: &FoundDevice, code: &str) -> anyhow::Result<DeviceProfile> {
    let key: String = rand::random::<[u8; 32]>().iter().map(|byte| format!("{byte:02x}")).collect();
    let client = DeviceClient::new(&found.address, Some(&found.fingerprint), None)?;
    client.post("adopt", &serde_json::json!({ "code": code.trim(), "key": key })).await?;
    crate::log_info!("Adopted the capture device {} at {}", found.info.id, found.address);
    Ok(DeviceProfile { id: found.info.id.clone(), address: found.address.clone(), fingerprint: found.fingerprint.clone(), key })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Against a real device, only reading: cargo test --lib capture_device -- --ignored, with CAPTURE_DEVICE set to its
    /// address.
    #[tokio::test]
    #[ignore]
    async fn finds_a_real_device() {
        let address = std::env::var("CAPTURE_DEVICE").expect("CAPTURE_DEVICE");
        let found = find(&address).await.unwrap();
        println!("{found:?}");
        let wrong = DeviceClient::new(&address, Some(&"0".repeat(64)), None).unwrap();
        let error = wrong.info().await.unwrap_err();
        println!("with the wrong fingerprint: {error:#}");
        assert!(format!("{error:#}").contains("not the one it had"));
    }
}
