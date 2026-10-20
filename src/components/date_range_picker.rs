//! A button showing a date range, which opens a dialog to change it.
//! Times are selected in half hour slots in a given timezone.

use dioxus::prelude::*;
use dioxus_free_icons::{Icon, icons::ld_icons::LdCalendar};
use sparko_graphql::types::{Date, DateTime};
use time_tz::{OffsetDateTimeExt, PrimitiveDateTimeExt, Tz};

/// The number of half hour slots in a day.
const SLOTS_PER_DAY: u8 = 48;

/// Format a half hour slot as HH:MM
fn slot_label(slot: u8) -> String {
    format!("{:02}:{:02}", slot / 2, (slot % 2) * 30)
}

/// A wall clock date and half hour time slot in the billing timezone.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct LocalSlot {
    pub date: time::Date,
    /// Half hours since midnight, 0..SLOTS_PER_DAY
    pub slot: u8,
}

impl LocalSlot {
    fn time(&self) -> time::Time {
        time::Time::from_hms(self.slot / 2, (self.slot % 2) * 30, 0).unwrap_or(time::Time::MIDNIGHT)
    }

    /// None if this wall clock time does not exist in the timezone, i.e. when the clocks go forward.
    /// When the clocks go back the earlier of the two possible times is used.
    pub fn to_date_time(&self, tz: &Tz) -> Option<DateTime> {
        let offset_date_time = time::PrimitiveDateTime::new(self.date, self.time()).assume_timezone(tz).take_first()?;
        DateTime::from_unix_timestamp(offset_date_time.unix_timestamp()).ok().map(|date_time| date_time.to_timezone(tz))
    }
}

impl std::fmt::Display for LocalSlot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", self.date, slot_label(self.slot))
    }
}

/// The selected period, an end of None means up to the present.
#[derive(Clone, Copy, PartialEq)]
pub struct RangeSelection {
    pub start: LocalSlot,
    pub end: Option<LocalSlot>,
}

impl RangeSelection {
    /// From the start of the current month in the given timezone, up to now.
    pub fn current_month(tz: &Tz) -> Self {
        let today = time::OffsetDateTime::now_utc().to_timezone(tz).date();
        RangeSelection {
            start: LocalSlot { date: today.replace_day(1).unwrap_or(today), slot: 0 },
            end: None,
        }
    }

    /// The start and exclusive end of the range, up to now is returned as an end of midnight at the start of tomorrow.
    pub fn to_date_times(&self, tz: &'static Tz) -> Result<(DateTime, DateTime), String> {
        let not_exist = |slot: &LocalSlot| format!("{slot} does not exist because the clocks go forward");

        let start = self.start.to_date_time(tz).ok_or_else(|| not_exist(&self.start))?;
        let end = match &self.end {
            Some(end_slot) => end_slot.to_date_time(tz).ok_or_else(|| not_exist(end_slot))?,
            None => Date::today(tz).next_day().at_midnight(tz),
        };
        if end <= start {
            return Err(String::from("The to time must be after the from time"));
        }
        Ok((start, end))
    }
}

impl std::fmt::Display for RangeSelection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.end {
            Some(end) => write!(f, "{} to {}", self.start, end),
            None => write!(f, "{} to now", self.start),
        }
    }
}

/// The values in the date range dialog, which may be incomplete while being edited.
#[derive(Clone)]
struct RangeDraft {
    start_date: Option<time::Date>,
    start_slot: u8,
    end_date: Option<time::Date>,
    end_slot: u8,
    to_now: bool,
}

impl RangeDraft {
    fn new(selection: &RangeSelection) -> Self {
        // When there is no end, offer the start date as a starting point for the user to edit.
        let end = selection.end.unwrap_or(selection.start);
        Self {
            start_date: Some(selection.start.date),
            start_slot: selection.start.slot,
            end_date: Some(end.date),
            end_slot: end.slot,
            to_now: selection.end.is_none(),
        }
    }

    fn to_selection(&self, tz: &'static Tz) -> Result<RangeSelection, String> {
        let start_date = self.start_date.ok_or_else(|| String::from("Select a from date"))?;
        let end = if self.to_now {
            None
        }
        else {
            let end_date = self.end_date.ok_or_else(|| String::from("Select a to date, or tick Up to now"))?;
            Some(LocalSlot { date: end_date, slot: self.end_slot })
        };
        let selection = RangeSelection { start: LocalSlot { date: start_date, slot: self.start_slot }, end };

        // Check that the times exist and are in the right order.
        selection.to_date_times(tz)?;
        Ok(selection)
    }
}

/// An empty or unparsable value clears the date.
fn parse_date_input(value: String) -> Option<time::Date> {
    value.parse::<Date>().ok().map(|date| *date)
}

/// Select element offering each half hour of the day.
fn slot_select(selected: u8, disabled: bool, onchange: impl FnMut(FormEvent) + 'static) -> Element {
    rsx! {
        select { disabled, onchange,
            for slot in 0..SLOTS_PER_DAY {
                option { value: "{slot}", selected: slot == selected, {slot_label(slot)} }
            }
        }
    }
}

#[derive(Props, Clone)]
pub struct DateRangePickerProps {
    /// The applied range, which is updated when the user applies a change.
    range: Signal<RangeSelection>,
    timezone: &'static Tz,
}

impl PartialEq for DateRangePickerProps {
    fn eq(&self, other: &Self) -> bool {
        self.range == other.range
            && std::ptr::eq(self.timezone, other.timezone)
    }
}

#[component]
pub fn DateRangePicker(props: DateRangePickerProps) -> Element {
    let tz = props.timezone;
    let mut range = props.range;

    // The contents of the date range dialog, Some while the dialog is open.
    let mut draft = use_signal::<Option<RangeDraft>>(|| None);

    let selection = *range.read();

    let dialog = if let Some(current) = draft.read().clone() {
        let validated = current.to_selection(tz);
        let draft_error = validated.as_ref().err().cloned();
        let can_apply = validated.is_ok();
        let start_value = current.start_date.map(|d| d.to_string()).unwrap_or_default();
        let end_value = current.end_date.map(|d| d.to_string()).unwrap_or_default();

        // Apply a change to the draft.
        let mut edit = move |change: &dyn Fn(&mut RangeDraft)| {
            if let Some(d) = draft.write().as_mut() {
                change(d);
            }
        };

        rsx! {
            div { class: "modal-backdrop",
                form {
                    class: "modal",
                    onsubmit: move |event: FormEvent| {
                        event.prevent_default();
                        if let Ok(selection) = &validated {
                            range.set(*selection);
                            draft.set(None);
                        }
                    },
                    h3 { "Select period" }
                    div { class: "modal-row",
                        label { "From" }
                        input {
                            r#type: "date",
                            value: "{start_value}",
                            onchange: move |event| edit(&|d| d.start_date = parse_date_input(event.value())),
                        }
                        {
                            slot_select(
                                current.start_slot,
                                false,
                                move |event| {
                                    if let Ok(slot) = event.value().parse::<u8>() {
                                        edit(&|d| d.start_slot = slot)
                                    }
                                },
                            )
                        }
                    }
                    div { class: "modal-row",
                        label { "To" }
                        input {
                            r#type: "date",
                            value: "{end_value}",
                            disabled: current.to_now,
                            onchange: move |event| edit(&|d| d.end_date = parse_date_input(event.value())),
                        }
                        {
                            slot_select(
                                current.end_slot,
                                current.to_now,
                                move |event| {
                                    if let Ok(slot) = event.value().parse::<u8>() {
                                        edit(&|d| d.end_slot = slot)
                                    }
                                },
                            )
                        }
                    }
                    div { class: "modal-row",
                        label {}
                        label {
                            input {
                                r#type: "checkbox",
                                checked: current.to_now,
                                onchange: move |event| edit(&|d| d.to_now = event.checked()),
                            }
                            " Up to now"
                        }
                    }
                    if let Some(error) = draft_error {
                        div { class: "modal-error", "{error}" }
                    }
                    div { class: "modal-buttons",
                        button {
                            r#type: "button",
                            onclick: move |_| draft.set(None),
                            "Cancel"
                        }
                        button { r#type: "submit", disabled: !can_apply, "Apply" }
                    }
                }
            }
        }
    }
    else {
        rsx! {}
    };

    rsx! {
        button {
            r#type: "button",
            title: "Change the period",
            onclick: move |_| draft.set(Some(RangeDraft::new(&range.read()))),
            Icon {
                class: "inline-icon",
                icon: LdCalendar,
                width: 14,
                height: 14,
            }
            " {selection}"
        }
        {dialog}
    }
}
