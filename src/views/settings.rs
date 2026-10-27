//! The settings of the active profile which are the app's rather than a module's, see profile::Settings: how much of the
//! log is kept, and whether the log panel is shown.

use std::sync::Arc;

use dioxus::prelude::*;

use crate::MarcoSparkoContext;
use crate::logging::{self, LogRetention};
use crate::profile::Settings as ProfileSettings;

/// The defaults of each kind of retention, offered when it is chosen.
const DEFAULT_DAYS: u32 = 7;
const DEFAULT_MEGABYTES: u32 = 10;

#[component]
pub fn Settings() -> Element {
    let context_signal = use_context::<Signal<Option<Arc<MarcoSparkoContext>>>>();
    let mut settings = use_context::<Signal<ProfileSettings>>();
    let profile_name = context_signal.read().as_ref().map(|context| context.profile.active_profile.name.clone()).unwrap_or_default();

    // The settings being edited, until they are saved.
    let mut retention = use_signal(|| settings.read().log.retention);
    let mut days = use_signal(|| match settings.read().log.retention { LogRetention::Days { days } => days, _ => DEFAULT_DAYS });
    let mut megabytes = use_signal(|| match settings.read().log.retention { LogRetention::Size { megabytes } => megabytes, _ => DEFAULT_MEGABYTES });
    let mut status = use_signal(|| None::<Result<String, String>>);

    let chosen = |kind: &str| match (kind, retention()) {
        ("lastRun", LogRetention::LastRun) | ("days", LogRetention::Days { .. }) | ("size", LogRetention::Size { .. }) => true,
        _ => false,
    };
    let log_path = logging::log_path(&profile_name).map(|path| path.display().to_string()).unwrap_or_default();

    let save = {
        let profile_name = profile_name.clone();
        move |_| {
            let mut new_settings = settings.read().clone();
            new_settings.log.retention = match retention() {
                LogRetention::LastRun => LogRetention::LastRun,
                LogRetention::Days { .. } => LogRetention::Days { days: days().max(1) },
                LogRetention::Size { .. } => LogRetention::Size { megabytes: megabytes().max(1) },
            };
            match crate::profile::update_settings(&profile_name, &new_settings) {
                Ok(()) => {
                    logging::set_retention(new_settings.log.retention);
                    crate::log_info!("Saved the settings of profile {profile_name}");
                    settings.set(new_settings);
                    status.set(Some(Ok(String::from("Saved"))));
                },
                Err(error) => {
                    crate::log_error!("Failed to save the settings of profile {profile_name}: {error}");
                    status.set(Some(Err(format!("Failed to save the settings: {error}"))));
                },
            }
        }
    };

    rsx! {
        div { class: "settings-page",
            h1 { "Settings" }
            p { class: "note", "The settings of profile {profile_name}." }

            h2 { "Log" }
            p { class: "note", "The log records the important things the app does, e.g. the data it fetched and anything which failed. It is kept in {log_path}." }
            div { class: "settings-group",
                label {
                    input {
                        r#type: "radio",
                        name: "log-keep",
                        checked: chosen("lastRun"),
                        onchange: move |_| retention.set(LogRetention::LastRun),
                    }
                    " Keep only the latest run of the app"
                }
                label {
                    input {
                        r#type: "radio",
                        name: "log-keep",
                        checked: chosen("days"),
                        onchange: move |_| retention.set(LogRetention::Days { days: days() }),
                    }
                    " Keep the last "
                    input {
                        class: "settings-number",
                        r#type: "number",
                        min: "1",
                        value: "{days}",
                        disabled: !chosen("days"),
                        oninput: move |event| if let Ok(value) = event.value().parse() { days.set(value) },
                    }
                    " days"
                }
                label {
                    input {
                        r#type: "radio",
                        name: "log-keep",
                        checked: chosen("size"),
                        onchange: move |_| retention.set(LogRetention::Size { megabytes: megabytes() }),
                    }
                    " Keep up to "
                    input {
                        class: "settings-number",
                        r#type: "number",
                        min: "1",
                        value: "{megabytes}",
                        disabled: !chosen("size"),
                        oninput: move |event| if let Ok(value) = event.value().parse() { megabytes.set(value) },
                    }
                    " MB, removing the oldest entries"
                }
            }
            p { class: "note", "Keeping only the latest run, or a number of days, takes effect the next time the app starts, so that this run's entries are kept until then." }

            div { class: "settings-buttons",
                button { r#type: "button", onclick: save, "Save" }
                match status() {
                    Some(Ok(message)) => rsx! { span { class: "ok", "{message}" } },
                    Some(Err(message)) => rsx! { span { class: "invalid", "{message}" } },
                    None => rsx! {},
                }
            }
        }
    }
}
