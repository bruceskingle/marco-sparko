use dioxus::prelude::*;
use time::macros::format_description;
use time_tz::OffsetDateTimeExt;

use crate::components::error_text::ErrorText;
use crate::polling::{PollState, PollStatuses, poll_now, set_paused};

/// The time in the computer's time zone, or UTC if it can not be found, e.g. 14:32:50
fn shown_time(time: Option<time::OffsetDateTime>) -> String {
    let format = format_description!("[hour]:[minute]:[second]");
    match (time, time_tz::system::get_timezone().ok()) {
        (Some(time), Some(tz)) => time.to_timezone(tz).format(&format).unwrap_or_default(),
        (Some(time), None) => time.format(&format).unwrap_or_default(),
        (None, _) => String::new(),
    }
}

/// What the app's polling of each module is doing, see polling.rs.
#[component]
pub fn PollingPage() -> Element {
    let statuses = use_context::<PollStatuses>();
    let rows: Vec<_> = statuses.read().iter().map(|(module_id, status)| (module_id.clone(), status.clone())).collect();

    rsx! {
        h1 { "Polling" }
        p {
            "While the app is open it polls the modules whose data nothing else keeps, such as the Tesla gateway and the zappi's status. Each poll is in the log too."
        }
        if rows.is_empty() {
            p { "There is nothing to poll in this profile." }
        } else {
            PollingTable { rows, statuses }
        }
    }
}

#[component]
fn PollingTable(rows: Vec<(String, crate::polling::PollStatus)>, statuses: PollStatuses) -> Element {
    rsx! {
        table { class: "display",
            tr {
                th { "Module" }
                th { "State" }
                th { "Every" }
                th { "Last Poll" }
                th { "Result" }
                th { "Next Poll" }
                th { "Polls" }
                th { "Failures" }
                th {}
            }
            for (module_id , status) in rows {
                tr {
                    td { "{module_id}" }
                    td {
                        {
                            match &status.state {
                                PollState::Starting => "Starting".to_string(),
                                PollState::NotPolled(why) => format!("Not polled: {why}"),
                                PollState::Waiting => "Waiting".to_string(),
                                PollState::Polling => "Polling...".to_string(),
                                PollState::Paused => "Paused".to_string(),
                            }
                        }
                    }
                    td { {status.interval.map(|interval| format!("{} min", interval.as_secs() / 60)).unwrap_or_default()} }
                    td { {shown_time(status.last_time)} }
                    td {
                        match &status.last_result {
                            Some(Ok(summary)) => rsx! { "{summary}" },
                            Some(Err(error)) => rsx! { ErrorText { error: error.clone() } },
                            None => rsx! {},
                        }
                    }
                    td {
                        if status.state == PollState::Waiting {
                            {shown_time(status.next_time)}
                        }
                    }
                    td { class: "number", "{status.polls}" }
                    td { class: "number", "{status.failures}" }
                    td { class: "poll-buttons",
                        // Shown, but disabled, while polling, so that the buttons do not jump about.
                        if matches!(status.state, PollState::Waiting | PollState::Paused | PollState::Polling) {
                            button {
                                disabled: status.state == PollState::Polling,
                                onclick: {
                                    let module_id = module_id.clone();
                                    move |_| poll_now(statuses, &module_id)
                                },
                                "Poll Now"
                            }
                        }
                        if matches!(status.state, PollState::Waiting | PollState::Polling) {
                            button {
                                disabled: status.state == PollState::Polling,
                                onclick: {
                                    let module_id = module_id.clone();
                                    move |_| set_paused(statuses, &module_id, true)
                                },
                                "Pause"
                            }
                        }
                        if status.state == PollState::Paused {
                            button {
                                onclick: {
                                    let module_id = module_id.clone();
                                    move |_| set_paused(statuses, &module_id, false)
                                },
                                "Resume"
                            }
                        }
                    }
                }
            }
        }
    }
}
