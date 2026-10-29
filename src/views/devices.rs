//! The Capture Devices page: the ESP32s the profile has adopted, see capture_device.rs, with their state, settings,
//! restart and release, and adopting another.

use std::collections::BTreeMap;
use std::sync::Arc;

use dioxus::prelude::*;
use serde_json::{Map, Value, json};

use crate::MarcoSparkoContext;
use crate::capture_device::{self, DeviceClient, DeviceProfile, FoundDevice};
use crate::profile::Settings as ProfileSettings;

/// A setting changed on the page but not yet saved, by feature and setting name.
type Edits = BTreeMap<(String, String), Value>;

/// Saves the profile's devices, in the profile and for the pages which show them.
fn save_devices(profile_name: &str, settings: &mut Signal<ProfileSettings>, devices: Vec<DeviceProfile>) -> Result<(), String> {
    crate::profile::update_capture_devices(profile_name, &devices).map_err(|error| format!("Unable to save the devices, {error:#}"))?;
    settings.write().capture_devices = devices;
    Ok(())
}

fn status_line(status: Option<Result<String, String>>) -> Element {
    match status {
        Some(Ok(message)) => rsx! { p { class: "ok", "{message}" } },
        Some(Err(message)) => rsx! { p { class: "invalid", "{message}" } },
        None => rsx! {},
    }
}

fn up_time(seconds: u64) -> String {
    match seconds {
        0..120 => format!("{seconds} s"),
        120..7200 => format!("{} min", seconds / 60),
        7200..172800 => format!("{} h", seconds / 3600),
        _ => format!("{} days", seconds / 86400),
    }
}

#[component]
pub fn Devices() -> Element {
    let context_signal = use_context::<Signal<Option<Arc<MarcoSparkoContext>>>>();
    let settings = use_context::<Signal<ProfileSettings>>();
    let profile_name = context_signal.read().as_ref().map(|context| context.profile.active_profile.name.clone()).unwrap_or_default();
    let devices = settings.read().capture_devices.clone();

    rsx! {
        div { class: "settings-page",
            h1 { "Capture Devices" }
            p { class: "note",
                "Capture devices are small always-on ESP32 boards which fetch data while this computer sleeps. Once adopted, only this profile can change their settings."
            }
            if devices.is_empty() {
                p { "There are no capture devices in profile {profile_name} yet." }
            }
            for device in devices {
                DeviceCard { key: "{device.id}", device: device.clone(), profile_name: profile_name.clone() }
            }
            AddDevice { profile_name: profile_name.clone() }
        }
    }
}

/// One adopted device: what it says it is, and its buttons.
#[component]
fn DeviceCard(device: DeviceProfile, profile_name: String) -> Element {
    let mut settings = use_context::<Signal<ProfileSettings>>();
    let mut generation = use_signal(|| 0u32);
    let mut status = use_signal(|| None::<Result<String, String>>);
    let mut busy = use_signal(|| false);
    let mut show_settings = use_signal(|| false);
    // Release or Forget waiting for Yes or No.
    let mut confirming = use_signal(|| None::<&'static str>);

    let info = use_resource({
        let device = device.clone();
        move || {
            let device = device.clone();
            generation();
            async move { DeviceClient::for_device(&device)?.info().await }
        }
    });

    let remove = {
        let device = device.clone();
        let profile_name = profile_name.clone();
        move || {
            let devices: Vec<DeviceProfile> = settings.read().capture_devices.iter().filter(|other| other.id != device.id).cloned().collect();
            if let Err(error) = save_devices(&profile_name, &mut settings, devices) {
                status.set(Some(Err(error)));
            }
        }
    };

    let restart = {
        let device = device.clone();
        move |_| {
            let device = device.clone();
            spawn(async move {
                busy.set(true);
                let result = async { DeviceClient::for_device(&device)?.restart().await }.await;
                status.set(Some(match result {
                    Ok(()) => Ok("Restarting, it takes about 20 seconds".to_string()),
                    Err(error) => Err(format!("Unable to restart it, {error:#}")),
                }));
                busy.set(false);
            });
        }
    };

    let confirm = {
        let device = device.clone();
        let remove = remove.clone();
        move |_| {
            let action = confirming.take();
            let device = device.clone();
            let mut remove = remove.clone();
            spawn(async move {
                if action == Some("release") {
                    busy.set(true);
                    let result = async { DeviceClient::for_device(&device)?.release().await }.await;
                    busy.set(false);
                    if let Err(error) = result {
                        status.set(Some(Err(format!("Unable to release it, {error:#}. If it cannot be reached, Forget it, then hold its BOOT button for 5 seconds."))));
                        return;
                    }
                    crate::log_info!("Released the capture device {}", device.id);
                }
                remove();
            });
        }
    };

    rsx! {
        div { class: "device-card",
            h2 { "{device.id}" }
            table { class: "device-details",
                tr { th { "Address" } td { "{device.address}" } }
                match &*info.read() {
                    None => rsx! { tr { th { "State" } td { "Connecting..." } } },
                    Some(Err(error)) => rsx! { tr { th { "State" } td { class: "invalid", {format!("Not reachable, {error:#}")} } } },
                    Some(Ok(info)) => rsx! {
                        tr { th { "Board" } td { "{info.board}" } }
                        tr { th { "Firmware" } td { "{info.firmware.name} {info.firmware.version} on {info.platform.name} {info.platform.version}" } }
                        tr { th { "Running for" } td { {up_time(info.up_time_seconds)} } }
                        tr { th { "Features" } td {
                            {info.features.iter().map(|feature| format!("{}{}", feature.name, if feature.enabled { "" } else { " (off)" })).collect::<Vec<_>>().join(", ")}
                        } }
                        if !info.adopted {
                            tr { th { "State" } td { class: "invalid", "No longer adopted, its BOOT button may have been held. Forget it, then adopt it again." } }
                        }
                    },
                }
            }
            div { class: "settings-buttons",
                button { r#type: "button", disabled: busy(), onclick: move |_| show_settings.set(!show_settings()), if show_settings() { "Hide Settings" } else { "Settings" } }
                button { r#type: "button", disabled: busy(), onclick: move |_| generation += 1, "Refresh" }
                button { r#type: "button", disabled: busy(), onclick: restart, "Restart" }
                button { r#type: "button", disabled: busy(), onclick: move |_| confirming.set(Some("release")), "Release" }
                button { r#type: "button", disabled: busy(), onclick: move |_| confirming.set(Some("forget")), "Forget" }
            }
            match confirming() {
                Some(action) => rsx! {
                    div { class: "settings-buttons",
                        span {
                            if action == "release" {
                                "Release the device, so that its own /config page manages it again, and remove it from this profile?"
                            } else {
                                "Remove the device from this profile without releasing it? It stays adopted until its BOOT button is held for 5 seconds."
                            }
                        }
                        button { r#type: "button", onclick: confirm, "Yes" }
                        button { r#type: "button", onclick: move |_| confirming.set(None), "No" }
                    }
                },
                None => rsx! {},
            }
            {status_line(status())}
            if show_settings() {
                DeviceSettings { device: device.clone() }
            }
        }
    }
}

/// Finds a device by its address and adopts it with the code on its screen.
#[component]
fn AddDevice(profile_name: String) -> Element {
    let mut settings = use_context::<Signal<ProfileSettings>>();
    let mut address = use_signal(String::new);
    let mut found = use_signal(|| None::<FoundDevice>);
    let mut pairing = use_signal(|| false);
    let mut code = use_signal(String::new);
    let mut status = use_signal(|| None::<Result<String, String>>);
    let mut busy = use_signal(|| false);

    let find = move || {
        spawn(async move {
            busy.set(true);
            status.set(None);
            found.set(None);
            pairing.set(false);
            match capture_device::find(&address()).await {
                Ok(device) => {
                    if settings.read().capture_devices.iter().any(|known| known.id == device.info.id) {
                        status.set(Some(Err(format!("{} is already in this profile", device.info.id))));
                    } else if device.info.adopted {
                        status.set(Some(Err("Another app, or another profile, has adopted it. To adopt it here, hold its BOOT button for 5 seconds to release it, then Find it again.".to_string())));
                        found.set(Some(device));
                    } else {
                        found.set(Some(device));
                    }
                }
                Err(error) => status.set(Some(Err(format!("{error:#}")))),
            }
            busy.set(false);
        });
    };

    let start = move |_| {
        spawn(async move {
            let Some(device) = found() else { return };
            busy.set(true);
            match capture_device::start_pairing(&device).await {
                Ok(time) => {
                    pairing.set(true);
                    code.set(String::new());
                    status.set(Some(Ok(format!("The device is showing a code, for {} minutes", time.as_secs() / 60))));
                }
                Err(error) => status.set(Some(Err(format!("Unable to start pairing, {error:#}")))),
            }
            busy.set(false);
        });
    };

    let adopt = move || {
        let profile_name = profile_name.clone();
        spawn(async move {
            let Some(device) = found() else { return };
            busy.set(true);
            match capture_device::adopt(&device, &code()).await {
                Ok(adopted) => {
                    let mut devices = settings.read().capture_devices.clone();
                    devices.retain(|other| other.id != adopted.id);
                    devices.push(adopted);
                    match save_devices(&profile_name, &mut settings, devices) {
                        Ok(()) => {
                            // Its card, which appears above, says so.
                            status.set(None);
                            found.set(None);
                            pairing.set(false);
                            address.set(String::new());
                        }
                        Err(error) => status.set(Some(Err(format!("The device was adopted, but {error}. Hold its BOOT button for 5 seconds and adopt it again.")))),
                    }
                }
                Err(error) => status.set(Some(Err(format!("Not adopted, {error:#}")))),
            }
            busy.set(false);
        });
    };

    rsx! {
        h2 { "Add a Capture Device" }
        p { class: "note", "Enter the device's IP address, shown on its /config page or by your router." }
        div { class: "settings-buttons",
            input {
                r#type: "text",
                placeholder: "e.g. 192.168.1.50",
                value: "{address}",
                oninput: move |event| address.set(event.value()),
                // Enter is the same as Find.
                onkeydown: move |event: KeyboardEvent| {
                    if event.key() == Key::Enter && !busy() && !address().trim().is_empty() {
                        find();
                    }
                },
            }
            button { r#type: "button", disabled: busy() || address().trim().is_empty(), onclick: move |_| find(), "Find" }
        }
        if let Some(device) = found() {
            table { class: "device-details",
                tr { th { "Device" } td { "{device.info.id}" } }
                tr { th { "Board" } td { "{device.info.board}" } }
                tr { th { "Firmware" } td { "{device.info.firmware.name} {device.info.firmware.version}" } }
                tr { th { "Certificate" } td { {device.fingerprint[..8].to_string()} } }
            }
            if !device.info.adopted {
                if pairing() {
                    p { class: "note",
                        {format!("Type the code on the device's screen. The screen also shows Certificate {}: if it shows something else, something else may be in between, so do not adopt it.", &device.fingerprint[..8])}
                    }
                    div { class: "settings-buttons",
                        input {
                            r#type: "text",
                            placeholder: "123 456",
                            value: "{code}",
                            oninput: move |event| code.set(event.value()),
                            // Enter is the same as Adopt.
                            onkeydown: {
                                let adopt = adopt.clone();
                                move |event: KeyboardEvent| {
                                    if event.key() == Key::Enter && !busy() && !code().trim().is_empty() {
                                        adopt();
                                    }
                                }
                            },
                        }
                        button { r#type: "button", disabled: busy() || code().trim().is_empty(), onclick: { let adopt = adopt.clone(); move |_| adopt() }, "Adopt" }
                        button { r#type: "button", disabled: busy(), onclick: start, "New Code" }
                    }
                } else {
                    div { class: "settings-buttons",
                        button { r#type: "button", disabled: busy(), onclick: start, "Show Pairing Code" }
                    }
                }
            }
        }
        {status_line(status())}
    }
}

/// The device's settings, a form for each feature, from what /api/config says about them.
#[component]
fn DeviceSettings(device: DeviceProfile) -> Element {
    let mut generation = use_signal(|| 0u32);
    let mut edits = use_signal(Edits::new);
    let mut enabled_edits = use_signal(BTreeMap::<String, bool>::new);
    let mut status = use_signal(|| None::<Result<String, String>>);
    let mut busy = use_signal(|| false);
    let mut restart_needed = use_signal(|| false);

    let config = use_resource({
        let device = device.clone();
        move || {
            let device = device.clone();
            generation();
            async move { DeviceClient::for_device(&device)?.config().await }
        }
    });

    let save = {
        let device = device.clone();
        move |_| {
            let mut features = Map::new();
            for (feature, enabled) in enabled_edits.read().iter() {
                features.insert(feature.clone(), json!({ "enabled": enabled }));
            }
            for ((feature, name), value) in edits.read().iter() {
                let entry = features.entry(feature.clone()).or_insert_with(|| json!({}));
                entry["settings"][name] = value.clone();
            }
            if features.is_empty() {
                status.set(Some(Ok("Nothing has changed".to_string())));
                return;
            }
            let device = device.clone();
            spawn(async move {
                busy.set(true);
                let update = json!({ "features": features });
                let result = async { DeviceClient::for_device(&device)?.update_config(&update).await }.await;
                match result {
                    Ok(_) => {
                        edits.write().clear();
                        enabled_edits.write().clear();
                        restart_needed.set(true);
                        status.set(Some(Ok("Saved, they take effect when the device restarts".to_string())));
                        generation += 1;
                    }
                    Err(error) => status.set(Some(Err(format!("Not saved, {error:#}")))),
                }
                busy.set(false);
            });
        }
    };

    let restart = {
        let device = device.clone();
        move |_| {
            let device = device.clone();
            spawn(async move {
                busy.set(true);
                let result = async { DeviceClient::for_device(&device)?.restart().await }.await;
                match result {
                    Ok(()) => {
                        restart_needed.set(false);
                        status.set(Some(Ok("Restarting, it takes about 20 seconds".to_string())));
                    }
                    Err(error) => status.set(Some(Err(format!("Unable to restart it, {error:#}")))),
                }
                busy.set(false);
            });
        }
    };

    rsx! {
        div { class: "device-settings",
            match &*config.read() {
                None => rsx! { p { "Reading the settings..." } },
                Some(Err(error)) => rsx! { p { class: "invalid", {format!("Unable to read the settings, {error:#}")} } },
                Some(Ok(json)) => rsx! {
                    for feature in json["features"].as_array().cloned().unwrap_or_default() {
                        FeatureSettings { key: "{feature[\"name\"]}", feature, edits, enabled_edits }
                    }
                },
            }
            div { class: "settings-buttons",
                button { r#type: "button", disabled: busy(), onclick: save, "Save" }
                button { r#type: "button", disabled: busy(), onclick: move |_| { edits.write().clear(); enabled_edits.write().clear(); status.set(None); }, "Undo Changes" }
                if restart_needed() {
                    button { r#type: "button", disabled: busy(), onclick: restart, "Restart Now" }
                }
            }
            {status_line(status())}
        }
    }
}

/// One feature's settings, with whether it is enabled.
#[component]
fn FeatureSettings(feature: Value, edits: Signal<Edits>, enabled_edits: Signal<BTreeMap<String, bool>>) -> Element {
    let name = feature["name"].as_str().unwrap_or_default().to_string();
    let required = feature["enabled"] == json!("required");
    let enabled = enabled_edits.read().get(&name).copied().unwrap_or_else(|| feature["enabled"] == json!(true));
    let settings = feature["settings"].as_array().cloned().unwrap_or_default();

    rsx! {
        fieldset { class: "device-feature",
            legend { "{name}" }
            if required {
                p { class: "note", "Always enabled" }
            } else {
                label { class: "device-setting",
                    span { "Enabled" }
                    input {
                        r#type: "checkbox",
                        checked: enabled,
                        onchange: {
                            let name = name.clone();
                            move |event: FormEvent| { enabled_edits.write().insert(name.clone(), event.checked()); }
                        },
                    }
                }
            }
            for setting in settings {
                SettingField { key: "{setting[\"name\"]}", feature: name.clone(), setting, edits }
            }
        }
    }
}

/// One setting, with an input for its type. A secret is never shown, only whether it is set: typing replaces it, and
/// Clear Password clears an optional one.
#[component]
fn SettingField(feature: String, setting: Value, edits: Signal<Edits>) -> Element {
    let name = setting["name"].as_str().unwrap_or_default().to_string();
    let kind = setting["type"].as_str().unwrap_or_default().to_string();
    let required = setting["required"].as_bool().unwrap_or_default();
    let secret = setting["secret"].as_bool().unwrap_or_default();
    let key = (feature, name.clone());
    let edited = edits.read().get(&key).cloned();
    let value = edited.clone().unwrap_or_else(|| setting["value"].clone());
    let text = match &value {
        Value::String(text) => text.clone(),
        Value::Null => String::new(),
        value => value.to_string(),
    };
    // Sets the edit to what the input says, removing it if that is what the device has, or, for a secret, if it is
    // empty, which keeps what the device has.
    let set = {
        let key = key.clone();
        let original = setting["value"].clone();
        move |value: Value| {
            if value == original || (secret && value == json!("")) {
                edits.write().remove(&key);
            } else {
                edits.write().insert(key.clone(), value);
            }
        }
    };
    // Empty clears an optional setting, as on the device's /config page.
    let text_value = move |text: String| if text.trim().is_empty() { Value::Null } else { json!(text) };
    let max_length = setting["maxLength"].as_u64().map(|len| len.to_string()).unwrap_or_default();
    let changed = edited.is_some();

    rsx! {
        label { class: if changed { "device-setting changed" } else { "device-setting" },
            span { "{name}" }
            match kind.as_str() {
                "bool" => rsx! {
                    input {
                        r#type: "checkbox",
                        checked: value.as_bool().unwrap_or_default(),
                        onchange: { let mut set = set.clone(); move |event: FormEvent| set(json!(event.checked())) },
                    }
                },
                "int32" | "int64" => rsx! {
                    input {
                        r#type: "number",
                        value: "{text}",
                        placeholder: if required { "" } else { "Default" },
                        oninput: {
                            let mut set = set.clone();
                            move |event: FormEvent| set(event.value().trim().parse::<i64>().map(|number| json!(number)).unwrap_or(Value::Null))
                        },
                    }
                },
                "timeZone" => rsx! {
                    select {
                        onchange: { let mut set = set.clone(); move |event: FormEvent| set(json!(event.value())) },
                        for option in setting["options"].as_array().cloned().unwrap_or_default() {
                            option {
                                value: option.as_str().unwrap_or_default(),
                                selected: option == value,
                                {option.as_str().unwrap_or_default().to_string()}
                            }
                        }
                    }
                },
                "color" => rsx! {
                    input {
                        r#type: "color",
                        value: if text.is_empty() { "#000000".to_string() } else { text.to_lowercase() },
                        oninput: { let mut set = set.clone(); move |event: FormEvent| set(json!(event.value().to_uppercase())) },
                    }
                    if !required && !value.is_null() {
                        button { r#type: "button", onclick: { let mut set = set.clone(); move |_| set(Value::Null) }, "Default" }
                    }
                },
                _ if secret => {
                    let is_set = setting["isSet"].as_bool().unwrap_or_default();
                    let cleared = edited == Some(Value::Null);
                    let placeholder = if cleared { "Cleared" } else if is_set { "Set, leave empty to keep" } else { "Not set" };
                    rsx! {
                        input {
                            r#type: "password",
                            maxlength: "{max_length}",
                            placeholder,
                            value: if cleared { String::new() } else { text.clone() },
                            oninput: { let mut set = set.clone(); move |event: FormEvent| set(json!(event.value())) },
                        }
                        if is_set && !required {
                            button { r#type: "button", onclick: { let mut set = set.clone(); move |_| set(Value::Null) }, "Clear Password" }
                        }
                    }
                },
                _ => rsx! {
                    input {
                        r#type: "text",
                        maxlength: "{max_length}",
                        value: "{text}",
                        placeholder: if required { "" } else { "Default" },
                        oninput: { let mut set = set.clone(); move |event: FormEvent| set(text_value(event.value())) },
                    }
                    if let Some(description) = setting["description"].as_str() {
                        span { class: "note", "{description}" }
                    }
                },
            }
        }
    }
}
