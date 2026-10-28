//! Electricity imported and exported, combined from all the readings there are: the meters' half hourly readings, which
//! split the import into the rates it is billed at, e.g. Home Low Rate, but arrive a few days later, and the Home Mini's
//! readings every 10 seconds, which are up to date but not split, and only go back as far as they were saved, see
//! telemetry.rs.
//!
//! The energy of each half hour is from the Home Mini if it has all of the half hour's readings, as it has the detail
//! within the half hour, otherwise from the meters. The two agree to within 0.1% over a half hour. The Home Mini's
//! import is split into the labels of the import meter's reading for the half hour, in its proportions. That is exact
//! for the rates, as each half hour is billed at one rate. Within the half hour the car's share is placed as a block at
//! the charger's power, as the car charges at a constant power, and the rest is the house's, see split_half_hour. The
//! car can not be told apart from the house's other loads by the power alone, e.g. a home battery charging at night.
//!
//! Import in a half hour which the meter has no reading for yet, e.g. today, is estimated to be at the low rate if it is
//! in the tariff's cheap times, or on an Intelligent Octopus tariff, if the car was charging, as outside the cheap times
//! it only charges the car during dispatches, which are at the low rate, otherwise at the standard rate, see Estimate.
//! On other tariffs charging outside the cheap times is at the standard rate. The car's share is not estimated. If the
//! tariff's cheap times are not known the import is UNMETERED_LABEL.
//!
//! The thresholds for seeing the car charging, CHARGING_POWER_SHARE and CHARGING_READINGS, were chosen from one
//! household's readings, with a 7 kW charger, on Intelligent Octopus Go: over 855 half hours of October 2026 the
//! estimate agreed with the meter for all but 9, each of which were either a little charging at a low power or a short
//! high load in the house. A home battery charging outside the cheap times would be taken for the car.

use std::collections::BTreeMap;

use time_tz::{OffsetDateTimeExt, Tz};

use super::measurements::order_labels;
use super::telemetry::{self, READING_SECONDS, TelemetryPeriod, TelemetryPoint, TelemetryReading};

/// The labels of import which the meter has no reading for yet, at the rate it is estimated to be billed at, or if that
/// can not be estimated, at an unknown rate.
pub const LOW_ESTIMATED_LABEL: &str = "Home Low Rate (estimated)";
pub const STANDARD_ESTIMATED_LABEL: &str = "Home Standard Rate (estimated)";
pub const UNMETERED_LABEL: &str = "Not yet metered";
pub const EXPORT_LABEL: &str = "Export";

const HALF_HOUR: i64 = 1800;
/// A half hour has all of its Home Mini readings if none of them, nor its start or end, are further apart than this.
/// The meter reads about every 10.3 seconds, so about one 10 second slot in 35 has no reading.
const MAX_READING_GAP: i64 = 60;

/// A half hour is estimated to have had the car charging if at least this many readings are at its charging power.
const CHARGING_READINGS: usize = 3;
/// The charging power is this much of the charger's power, so that the house's load on top is not needed.
const CHARGING_POWER_SHARE: f64 = 0.85;

/// The times of a tariff's cheapest rate, in minutes from midnight, each from up to to, which may be past midnight,
/// e.g. 23:30 to 05:30 is (1410, 330).
#[derive(Clone, Debug, PartialEq)]
pub struct CheapTimes(Vec<(i64, i64)>);

impl CheapTimes {
    /// The cheap times from the timeslots of a tariff's time of use scheme, as names and times, e.g. ECO7_NIGHT from
    /// 23:30:00 to 05:30:00. The cheap ones are those named as night, off peak, low or cheap, or if none are, the
    /// shortest. None if there is only one timeslot, so only one rate, or the times can not be read.
    pub fn from_timeslots(timeslots: &[(String, String, String)]) -> Option<Self> {
        let minutes = |time: &str| -> Option<i64> {
            let mut parts = time.split(':');
            Some(parts.next()?.parse::<i64>().ok()? * 60 + parts.next()?.parse::<i64>().ok()?)
        };
        let slots: Vec<(String, i64, i64)> = timeslots.iter()
            .map(|(name, from, to)| Some((name.to_uppercase(), minutes(from)?, minutes(to)?)))
            .collect::<Option<_>>()?;
        if slots.len() < 2 {
            return None;
        }
        let named: Vec<(i64, i64)> = slots.iter()
            .filter(|(name, _, _)| ["NIGHT", "OFF_PEAK", "OFFPEAK", "LOW", "CHEAP"].iter().any(|word| name.contains(word)))
            .map(|(_, from, to)| (*from, *to))
            .collect();
        let length = |(from, to): &(i64, i64)| (to - from).rem_euclid(24 * 60);
        let cheap = if named.is_empty() {
            slots.iter().map(|(_, from, to)| (*from, *to)).min_by_key(length).into_iter().collect()
        }
        else {
            named
        };
        Some(Self(cheap))
    }

    /// Whether the minute from midnight is in the cheap times.
    fn contains(&self, minute: i64) -> bool {
        self.0.iter().any(|(from, to)| if from <= to { (*from..*to).contains(&minute) } else { minute >= *from || minute < *to })
    }
}

/// What is known about the tariff and the car's charger, for estimating the rate of import which the meter has no
/// reading for yet, and placing the car's share of the import, see the module comment.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Estimate {
    pub cheap: Option<CheapTimes>,
    /// Whether the tariff is an Intelligent Octopus one, on which charging outside the cheap times is at the low rate,
    /// see is_intelligent.
    pub intelligent: bool,
    /// The charger's power, in W, None if there is no charger or it is not known.
    pub charger_watts: Option<f64>,
}

/// Whether a tariff's product code is an Intelligent Octopus one, e.g. IOG-VAR-24-10-29 (Intelligent Octopus Go) or
/// INTELLI-VAR-22-10-14 (Intelligent Octopus).
pub fn is_intelligent(product_code: &str) -> bool {
    product_code.starts_with("IOG") || product_code.starts_with("INTELLI")
}

/// Where in a half hour the car's share of the import is placed, see split_half_hour.
#[derive(Clone, Copy, Debug, PartialEq)]
enum Placement {
    /// It was charging in the half hours before and after too.
    Throughout,
    /// It was charging in the half hour before, so it carried on from the start.
    Start,
    /// It was charging in the half hour after, so it started before the end.
    End,
    /// Where the import is highest, as it was charging in neither the half hour before nor the one after. This assumes
    /// the car is the largest load, which it may not be, e.g. with a home battery charging.
    Highest,
}

/// The units of each label of a meter's half hourly reading, in kWh, with its start and end as unix timestamps, see
/// MeasurementsDataSet::half_hours.
#[derive(Clone, Debug, PartialEq)]
pub struct MeterHalfHour {
    pub start: i64,
    pub end: i64,
    pub units: BTreeMap<String, f64>,
}

impl MeterHalfHour {
    fn total(&self) -> f64 {
        self.units.values().sum()
    }
}

/// The electricity in a period, in kWh, with its start and end as unix timestamps.
#[derive(Clone, Debug, PartialEq)]
pub struct ElectricityPoint {
    pub start: i64,
    pub end: i64,
    /// The energy imported, for each of Electricity::labels.
    pub import: Vec<f64>,
    pub export: f64,
    /// The Home Mini's average power, in W, net of export, for periods shorter than a half hour from its readings,
    /// which is steadier than their energy, as the readings are not exactly 10 seconds apart.
    pub demand: Option<f64>,
    /// Whether any of it is from the Home Mini, and whether any is from the meters.
    pub home_mini: bool,
    pub meter: bool,
}

/// The electricity in each period which has any, in time order, with the labels its import is split into.
#[derive(Clone, Debug, PartialEq)]
pub struct Electricity {
    pub labels: Vec<String>,
    pub points: Vec<ElectricityPoint>,
}

/// A period's electricity while it is worked out, with the import by label.
#[derive(Default)]
struct Period {
    start: i64,
    end: i64,
    import: BTreeMap<String, f64>,
    export: f64,
    /// The Home Mini's average power, in W, for periods shorter than a half hour.
    demand: Option<f64>,
    home_mini: bool,
    meter: bool,
}

impl Period {
    fn add(&mut self, other: Period) {
        self.demand = None;
        for (label, units) in other.import {
            *self.import.entry(label).or_default() += units;
        }
        self.export += other.export;
        self.home_mini |= other.home_mini;
        self.meter |= other.meter;
    }
}

/// The start of each half hour from start up to end, as unix timestamps, which the Home Mini has all the readings for,
/// see MAX_READING_GAP. readings are in time order. The half hour of the latest reading counts up to that reading, so
/// that the half hour in progress is shown.
fn complete_half_hours(readings: &[TelemetryReading], start: i64, end: i64) -> Vec<i64> {
    let Some(latest) = readings.last().map(|reading| reading.read_at + READING_SECONDS) else {
        return Vec::new();
    };
    let mut complete = Vec::new();
    let mut half_hour = start - start.rem_euclid(HALF_HOUR);
    while half_hour < end.min(latest) {
        let half_hour_end = (half_hour + HALF_HOUR).min(latest);
        let first = readings.partition_point(|reading| reading.read_at < half_hour);
        let mut previous = half_hour;
        let mut gap_ok = true;
        let mut any = false;
        for reading in readings[first..].iter().take_while(|reading| reading.read_at < half_hour_end) {
            gap_ok &= reading.read_at - previous <= MAX_READING_GAP;
            previous = reading.read_at;
            any = true;
        }
        if any && gap_ok && half_hour_end - previous <= MAX_READING_GAP {
            complete.push(half_hour);
        }
        half_hour += HALF_HOUR;
    }
    complete
}

/// The import of each of a half hour's points, kwh, which are seconds long, split into the labels of the import meter's
/// reading for it, units. The import is the Home Mini's, which is very close to the meter's, so the meter's proportions
/// are used rather than its units. The car's share (labels EV ...) is placed as a block at the charger's power, if it is
/// known, at the placement, and is otherwise shared in proportion to the points' import. The rest of each point's import
/// is the house's, in the proportions of the other labels.
fn split_half_hour(kwh: &[f64], seconds: &[i64], units: &BTreeMap<String, f64>, charger_watts: Option<f64>, placement: Placement)
    -> Vec<BTreeMap<String, f64>>
{
    let meter_total: f64 = units.values().sum();
    let is_ev = |label: &str| label.starts_with("EV");
    let ev_units: f64 = units.iter().filter(|(label, _)| is_ev(label)).map(|(_, units)| units).sum();
    let home_units = meter_total - ev_units;
    let total: f64 = kwh.iter().sum();
    let ev_total = total * ev_units / meter_total;

    // The most of each point which can be the car's.
    let room: Vec<f64> = match charger_watts {
        Some(watts) => kwh.iter().zip(seconds).map(|(kwh, seconds)| kwh.min(watts * *seconds as f64 / 3_600_000.0)).collect(),
        None => kwh.to_vec(),
    };
    let room_total: f64 = room.iter().sum();
    let mut ev = vec![0.0; kwh.len()];
    let fill = |ev: &mut Vec<f64>, order: &mut dyn Iterator<Item = usize>| {
        let mut left = ev_total;
        for index in order {
            ev[index] = room[index].min(left);
            left -= ev[index];
        }
    };
    match (charger_watts, placement) {
        (None, _) | (_, Placement::Throughout) => if room_total > 0.0 {
            let share = (ev_total / room_total).min(1.0);
            for (ev, room) in ev.iter_mut().zip(&room) {
                *ev = room * share;
            }
        },
        (_, Placement::Start) => fill(&mut ev, &mut (0..kwh.len())),
        (_, Placement::End) => fill(&mut ev, &mut (0..kwh.len()).rev()),
        (_, Placement::Highest) => {
            // The run of points long enough for the car's share with the most import, then on from there.
            let average_room = room_total / room.len().max(1) as f64;
            let length = if average_room > 0.0 { ((ev_total / average_room).ceil() as usize).clamp(1, kwh.len().max(1)) } else { 1 };
            let first = (0..=kwh.len().saturating_sub(length))
                .max_by(|a, b| kwh[*a..*a + length].iter().sum::<f64>().total_cmp(&kwh[*b..*b + length].iter().sum::<f64>()))
                .unwrap_or_default();
            fill(&mut ev, &mut (first..kwh.len()).chain(0..first));
        },
    }
    // Anything which did not fit, e.g. as the charger's power is a little low, is shared by the points with room.
    let left_over = ev_total - ev.iter().sum::<f64>();
    let space: f64 = kwh.iter().zip(&ev).map(|(kwh, ev)| kwh - ev).sum();
    if left_over > 1e-12 && space > 0.0 {
        for (ev, kwh) in ev.iter_mut().zip(kwh) {
            *ev += left_over * (kwh - *ev) / space;
        }
    }

    kwh.iter().zip(ev)
        .map(|(kwh, ev)| {
            let home = kwh - ev;
            units.iter()
                .map(|(label, units)| {
                    let share = match (is_ev(label), ev_units > 0.0, home_units > 0.0) {
                        (true, true, _) => ev * units / ev_units,
                        (false, _, true) => home * units / home_units,
                        (true, false, _) | (false, _, false) => 0.0,
                    };
                    (label.clone(), share)
                })
                // Import in a point which the meter gives none of to the house is given to the car.
                .chain((home_units == 0.0 && home > 0.0).then(|| {
                    let label = units.keys().find(|label| is_ev(label)).cloned().unwrap_or_default();
                    (label, home)
                }))
                .fold(BTreeMap::new(), |mut labels, (label, kwh)| { *labels.entry(label).or_insert(0.0) += kwh; labels })
        })
        .collect()
}

/// The points with each half hour's import and export shared out between its points in proportion to the Home Mini's
/// power readings, rather than the changes in its registers between readings. The registers' totals are exact, but the
/// energy between two readings varies by several percent with the time between them, which is about 10.3 seconds
/// rather than the 10 their times say, and as the registers count whole Wh, which over 10 seconds looks like the power
/// changing when it is steady. A half hour with a point without a power reading is left as it is.
fn shaped_by_power(mut points: Vec<TelemetryPoint>) -> Vec<TelemetryPoint> {
    // Each point's energy in Wh at its power reading, import from positive power and export from negative.
    let at_power = |point: &TelemetryPoint, sign: f64| (sign * point.demand.unwrap_or_default()).max(0.0) * (point.end - point.start) as f64 / 3600.0;
    let half_hour = |point: &TelemetryPoint| point.start - point.start.rem_euclid(HALF_HOUR);
    for group in points.chunk_by_mut(|a, b| half_hour(a) == half_hour(b)) {
        if group.iter().any(|point| point.demand.is_none()) {
            continue;
        }
        let (import, export): (f64, f64) = group.iter().fold((0.0, 0.0), |(import, export), point| (import + point.energy, export + point.export));
        let (import_power, export_power): (f64, f64) = group.iter()
            .fold((0.0, 0.0), |(import, export), point| (import + at_power(point, 1.0), export + at_power(point, -1.0)));
        for point in group.iter_mut() {
            if import_power > 0.0 {
                point.energy = import * at_power(point, 1.0) / import_power;
            }
            if export_power > 0.0 {
                point.export = export * at_power(point, -1.0) / export_power;
            }
        }
    }
    points
}

/// The start of each half hour, as unix timestamps, in which at least CHARGING_READINGS readings are at or above the
/// charging power, in W.
fn charging_half_hours(readings: &[TelemetryReading], watts: f64) -> std::collections::BTreeSet<i64> {
    let mut counts: BTreeMap<i64, usize> = BTreeMap::new();
    for reading in readings.iter().filter(|reading| reading.demand.is_some_and(|demand| demand >= watts)) {
        *counts.entry(reading.read_at - reading.read_at.rem_euclid(HALF_HOUR)).or_default() += 1;
    }
    counts.into_iter().filter(|(_, count)| *count >= CHARGING_READINGS).map(|(half_hour, _)| half_hour).collect()
}

/// The electricity from start to end, as unix timestamps, in periods of period, from the import and export meters'
/// half hours and the Home Mini's readings, see the module comment. Periods shorter than a half hour are only made from
/// the Home Mini's readings, half hours which only the meters have are shown as a half hour.
pub fn combine(import: &[MeterHalfHour], export: &[MeterHalfHour], readings: &[TelemetryReading], start: i64, end: i64,
    period: TelemetryPeriod, estimate: &Estimate, tz: &'static Tz) -> Electricity
{
    let import_meter: BTreeMap<i64, &MeterHalfHour> = import.iter().map(|half_hour| (half_hour.start, half_hour)).collect();
    let export_meter: BTreeMap<i64, f64> = export.iter().map(|half_hour| (half_hour.start, half_hour.total())).collect();
    let complete: std::collections::BTreeSet<i64> = complete_half_hours(readings, start, end).into_iter().collect();
    let charging = estimate.charger_watts
        .map(|watts| charging_half_hours(readings, watts * CHARGING_POWER_SHARE))
        .unwrap_or_default();
    let charged = |half_hour: i64| import_meter.get(&half_hour)
        .is_some_and(|reading| reading.units.iter().any(|(label, units)| label.starts_with("EV") && *units > 0.0));

    // Whether the import in the half hour starting at half_hour, which the meter has no reading for, is estimated to be at
    // the low rate, None if it can not be estimated.
    let estimated_low = |half_hour: i64| -> Option<bool> {
        let cheap = estimate.cheap.as_ref()?;
        let minute = time::OffsetDateTime::from_unix_timestamp(half_hour)
            .map(|time| time.to_timezone(tz))
            .map(|time| time.hour() as i64 * 60 + time.minute() as i64)
            .unwrap_or_default();
        Some(cheap.contains(minute) || (estimate.intelligent && charging.contains(&half_hour)))
    };

    // The Home Mini's readings in the half hours it has them all for, in periods no longer than a half hour.
    let detail = match period.seconds() {
        Some(seconds) if seconds <= HALF_HOUR => period,
        _ => TelemetryPeriod::HalfHour,
    };
    let points: Vec<TelemetryPoint> = telemetry::aggregate(readings, start, end, detail, tz).into_iter()
        .filter(|point| point.readings > 0 && complete.contains(&(point.start - point.start.rem_euclid(HALF_HOUR))))
        .collect();
    let points = shaped_by_power(points);

    // Each half hour's points, with their import split into labels, see split_half_hour.
    let mut parts: Vec<Period> = Vec::new();
    for half_hour_points in points.chunk_by(|a, b| a.start - a.start.rem_euclid(HALF_HOUR) == b.start - b.start.rem_euclid(HALF_HOUR)) {
        let half_hour = half_hour_points[0].start - half_hour_points[0].start.rem_euclid(HALF_HOUR);
        let kwh: Vec<f64> = half_hour_points.iter().map(|point| point.energy / 1000.0).collect();
        let seconds: Vec<i64> = half_hour_points.iter().map(|point| point.end - point.start).collect();
        let placement = match (charged(half_hour - HALF_HOUR), charged(half_hour + HALF_HOUR)) {
            (true, true) => Placement::Throughout,
            (true, false) => Placement::Start,
            (false, true) => Placement::End,
            (false, false) => Placement::Highest,
        };
        let labels = match import_meter.get(&half_hour).filter(|reading| reading.total() > 0.0) {
            Some(reading) => split_half_hour(&kwh, &seconds, &reading.units, estimate.charger_watts, placement),
            None => {
                let label = match estimated_low(half_hour) {
                    Some(true) => LOW_ESTIMATED_LABEL,
                    Some(false) => STANDARD_ESTIMATED_LABEL,
                    None => UNMETERED_LABEL,
                };
                kwh.iter().map(|kwh| BTreeMap::from([(String::from(label), *kwh)])).collect()
            },
        };
        parts.extend(half_hour_points.iter().zip(labels).map(|(point, import)| Period {
            start: point.start,
            end: point.end,
            import,
            export: point.export / 1000.0,
            demand: point.demand,
            home_mini: true,
            meter: false,
        }));
    }

    // The meters' readings of the other half hours.
    let mut half_hour = start - start.rem_euclid(HALF_HOUR);
    while half_hour < end {
        if !complete.contains(&half_hour) {
            let reading = import_meter.get(&half_hour);
            let exported = export_meter.get(&half_hour);
            if reading.is_some() || exported.is_some() {
                parts.push(Period {
                    start: half_hour,
                    end: half_hour + HALF_HOUR,
                    import: reading.map(|reading| reading.units.clone()).unwrap_or_default(),
                    export: exported.copied().unwrap_or_default(),
                    demand: None,
                    home_mini: false,
                    meter: true,
                });
            }
        }
        half_hour += HALF_HOUR;
    }

    // Periods of a half hour or longer add the parts in each together.
    let periods: Vec<Period> = if detail == period && period != TelemetryPeriod::HalfHour {
        parts.sort_by_key(|part| part.start);
        parts
    }
    else {
        let mut periods: BTreeMap<i64, Period> = BTreeMap::new();
        for part in parts {
            let (period_start, period_end) = telemetry::period_bounds(part.start, period, tz);
            periods.entry(period_start)
                .or_insert_with(|| Period { start: period_start, end: period_end, ..Period::default() })
                .add(part);
        }
        periods.into_values().collect()
    };

    let labels = order_labels(periods.iter().flat_map(|period| period.import.keys().map(String::as_str)));
    let points = periods.into_iter()
        .map(|period| ElectricityPoint {
            start: period.start,
            end: period.end,
            import: labels.iter().map(|label| period.import.get(label).copied().unwrap_or_default()).collect(),
            export: period.export,
            demand: period.demand,
            home_mini: period.home_mini,
            meter: period.meter,
        })
        .collect();
    Electricity { labels, points }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::{CheapTimes, EXPORT_LABEL, Estimate, LOW_ESTIMATED_LABEL, MeterHalfHour, Placement, STANDARD_ESTIMATED_LABEL,
        UNMETERED_LABEL, combine, complete_half_hours, is_intelligent, shaped_by_power, split_half_hour};
    use crate::octopus::telemetry::TelemetryPoint;
    use crate::octopus::telemetry::{TelemetryPeriod, TelemetryReading};

    const LONDON: &time_tz::Tz = time_tz::timezones::db::europe::LONDON;
    /// Midnight UTC, in GMT, so also local midnight.
    const DAY: i64 = 1_793_232_000;

    /// A reading every 10 seconds from start up to end, importing 10 Wh and exporting 1 Wh each, with the registers
    /// starting at the given values.
    fn readings(start: i64, end: i64) -> Vec<TelemetryReading> {
        (start..end).step_by(10)
            .enumerate()
            .map(|(index, read_at)| TelemetryReading {
                read_at,
                demand: Some(3600.0),
                consumption_delta: Some(10.0),
                cost_delta: None,
                cost_delta_with_tax: None,
                consumption: Some(1000.0 + 10.0 * index as f64),
                export: Some(500.0 + index as f64),
            })
            .collect()
    }

    fn half_hour(start: i64, units: &[(&str, f64)]) -> MeterHalfHour {
        MeterHalfHour {
            start,
            end: start + 1800,
            units: units.iter().map(|(label, units)| (label.to_string(), *units)).collect(),
        }
    }

    #[test]
    fn a_half_hour_has_all_its_readings_if_none_are_more_than_a_minute_apart() {
        let mut all = readings(DAY - 10, DAY + 7200);
        assert_eq!(complete_half_hours(&all, DAY, DAY + 7200), [DAY, DAY + 1800, DAY + 3600, DAY + 5400]);
        // Two minutes missing from the second half hour, and the last half hour only up to its latest reading.
        all.retain(|reading| !(DAY + 2000..DAY + 2120).contains(&reading.read_at) && reading.read_at < DAY + 6000);
        assert_eq!(complete_half_hours(&all, DAY, DAY + 7200), [DAY, DAY + 3600, DAY + 5400]);
        assert!(complete_half_hours(&[], DAY, DAY + 7200).is_empty());
    }

    #[test]
    fn the_home_mini_is_used_where_it_is_complete_split_by_the_meters_labels_and_the_meters_elsewhere() {
        // The Home Mini has the second half hour, the meters the first two, and nothing has the third.
        let all = readings(DAY + 1790, DAY + 3600);
        let import = [
            half_hour(DAY, &[("Home Standard Rate", 2.0)]),
            half_hour(DAY + 1800, &[("Home Low Rate", 1.0), ("EV Low Rate", 3.0)]),
        ];
        let export = [half_hour(DAY, &[("GENERATION", 0.5)]), half_hour(DAY + 1800, &[("GENERATION", 0.25)])];
        let electricity = combine(&import, &export, &all, DAY, DAY + 5400, TelemetryPeriod::HalfHour, &Estimate::default(), LONDON);
        assert_eq!(electricity.labels, ["Home Low Rate", "Home Standard Rate", "EV Low Rate"]);
        let points = &electricity.points;
        assert_eq!(points.len(), 2);
        assert_eq!((points[0].start, points[0].import.clone(), points[0].export), (DAY, vec![0.0, 2.0, 0.0], 0.5));
        assert!(points[0].meter && !points[0].home_mini);
        // 180 readings of 10 Wh, split 1 to 3 as the meter's reading is, and 180 of 1 Wh exported.
        assert_eq!(points[1].start, DAY + 1800);
        assert!((points[1].import[0] - 0.45).abs() < 1e-9 && (points[1].import[2] - 1.35).abs() < 1e-9);
        assert!((points[1].export - 0.18).abs() < 1e-9);
        assert!(points[1].home_mini && !points[1].meter);
    }

    #[test]
    fn import_the_meter_has_not_read_yet_is_unmetered_and_shorter_periods_are_from_the_home_mini() {
        let all = readings(DAY - 10, DAY + 1800);
        let electricity = combine(&[], &[], &all, DAY, DAY + 1800, TelemetryPeriod::TenMinutes, &Estimate::default(), LONDON);
        assert_eq!(electricity.labels, [UNMETERED_LABEL]);
        let starts: Vec<i64> = electricity.points.iter().map(|point| point.start).collect();
        assert_eq!(starts, [DAY, DAY + 600, DAY + 1200]);
        assert!(electricity.points.iter().all(|point| (point.import[0] - 0.6).abs() < 1e-9));
        assert_ne!(EXPORT_LABEL, UNMETERED_LABEL);
    }

    #[test]
    fn a_day_adds_the_home_mini_and_the_meters_half_hours_together() {
        let all = readings(DAY + 1790, DAY + 3600);
        let import = [half_hour(DAY, &[("Home Low Rate", 2.0)]), half_hour(DAY + 1800, &[("Home Low Rate", 9.0)])];
        let electricity = combine(&import, &[], &all, DAY, DAY + 86400, TelemetryPeriod::Day, &Estimate::default(), LONDON);
        assert_eq!(electricity.points.len(), 1);
        let day = &electricity.points[0];
        assert_eq!((day.start, day.end), (DAY, DAY + 86400));
        // The meter's 2 kWh and the Home Mini's 1.8 kWh, rather than the meter's 9 kWh for the second half hour.
        assert!((day.import[0] - 3.8).abs() < 1e-9);
        assert!(day.home_mini && day.meter);
    }

    fn slots(slots: &[(&str, &str, &str)]) -> Vec<(String, String, String)> {
        slots.iter().map(|(name, from, to)| (name.to_string(), from.to_string(), to.to_string())).collect()
    }

    #[test]
    fn the_cheap_times_are_the_night_timeslot_or_else_the_shortest() {
        let go = CheapTimes::from_timeslots(&slots(&[("ECO7_DAY", "05:30:00", "23:30:00"), ("ECO7_NIGHT", "23:30:00", "05:30:00")]));
        assert_eq!(go, Some(CheapTimes(vec![(1410, 330)])));
        let go = go.unwrap();
        assert!(go.contains(23 * 60 + 30) && go.contains(0) && go.contains(5 * 60) && !go.contains(5 * 60 + 30) && !go.contains(12 * 60));
        let unnamed = CheapTimes::from_timeslots(&slots(&[("A", "07:00", "02:00"), ("B", "02:00", "07:00")]));
        assert_eq!(unnamed, Some(CheapTimes(vec![(120, 420)])));
        assert_eq!(CheapTimes::from_timeslots(&slots(&[("ALL", "00:00", "00:00")])), None);
    }

    #[test]
    fn unmetered_import_is_low_rate_in_the_cheap_times_or_while_charging_and_otherwise_standard() {
        let cheap = CheapTimes::from_timeslots(&slots(&[("ECO7_DAY", "05:30", "23:30"), ("ECO7_NIGHT", "23:30", "05:30")])).unwrap();
        let estimate = Estimate { cheap: Some(cheap), intelligent: true, charger_watts: Some(7000.0) };
        // From 05:00 to 06:30, with the car charging at 7 kW from 06:00.
        let mut all = readings(DAY + 5 * 3600 - 10, DAY + 6 * 3600 + 1800);
        for reading in all.iter_mut().filter(|reading| reading.read_at >= DAY + 6 * 3600) {
            reading.demand = Some(7000.0);
        }
        let electricity = combine(&[], &[], &all, DAY + 5 * 3600, DAY + 6 * 3600 + 1800, TelemetryPeriod::HalfHour, &estimate, LONDON);
        assert_eq!(electricity.labels, [LOW_ESTIMATED_LABEL, STANDARD_ESTIMATED_LABEL]);
        let rates: Vec<Vec<bool>> = electricity.points.iter().map(|point| point.import.iter().map(|kwh| *kwh > 0.0).collect()).collect();
        assert_eq!(rates, [vec![true, false], vec![false, true], vec![true, false]]);
        // On a tariff which is not an Intelligent Octopus one, charging outside the cheap times is at the standard rate.
        let other = Estimate { intelligent: false, ..estimate };
        let electricity = combine(&[], &[], &all, DAY + 5 * 3600, DAY + 6 * 3600 + 1800, TelemetryPeriod::HalfHour, &other, LONDON);
        let rates: Vec<Vec<bool>> = electricity.points.iter().map(|point| point.import.iter().map(|kwh| *kwh > 0.0).collect()).collect();
        assert_eq!(rates, [vec![true, false], vec![false, true], vec![false, true]]);
        assert!(is_intelligent("IOG-VAR-24-10-29") && is_intelligent("INTELLI-VAR-22-10-14") && !is_intelligent("GO-VAR-22-10-14"));
    }

    #[test]
    fn the_cars_share_of_a_metered_half_hour_is_a_block_at_the_chargers_power_where_it_was_charging() {
        // Six 10 second points, the car's share 0.06 kWh, which is three points at 7.2 kW (0.02 kWh each).
        let units = BTreeMap::from([(String::from("EV Low Rate"), 1.0), (String::from("Home Low Rate"), 4.0)]);
        let seconds = [10; 6];
        let ev = |kwh: &[f64], placement: Placement| -> Vec<f64> {
            split_half_hour(kwh, &seconds, &units, Some(7200.0), placement).iter().map(|labels| labels["EV Low Rate"]).collect()
        };
        let close = |actual: Vec<f64>, expected: [f64; 6]| actual.iter().zip(expected).all(|(actual, expected)| (actual - expected).abs() < 1e-9);
        let flat = [0.05; 6];
        assert!(close(ev(&flat, Placement::Start), [0.02, 0.02, 0.02, 0.0, 0.0, 0.0]));
        assert!(close(ev(&flat, Placement::End), [0.0, 0.0, 0.0, 0.02, 0.02, 0.02]));
        assert!(close(ev(&flat, Placement::Throughout), [0.01; 6]));
        // Where the import is highest, as the car adds its power to the house's: its 0.048 kWh share of 0.24 kWh from the
        // first of the three highest points.
        let step = [0.03, 0.03, 0.05, 0.05, 0.05, 0.03];
        assert!(close(ev(&step, Placement::Highest), [0.0, 0.0, 0.02, 0.02, 0.008, 0.0]));
        // Every point's labels add up to its import.
        let split = split_half_hour(&step, &seconds, &units, Some(7200.0), Placement::Highest);
        assert!(split.iter().zip(step).all(|(labels, kwh)| (labels.values().sum::<f64>() - kwh).abs() < 1e-9));
    }

    #[test]
    fn a_half_hours_energy_is_shared_out_by_the_power_readings() {
        let point = |start: i64, energy: f64, demand: f64| TelemetryPoint { start, end: start + 10, energy, export: 0.0, demand: Some(demand), readings: 1 };
        // Steady power, but the energy between readings jittering, and a total of 120 Wh.
        let shaped = shaped_by_power(vec![point(DAY, 39.0, 14000.0), point(DAY + 10, 43.0, 14000.0), point(DAY + 20, 38.0, 14000.0)]);
        assert!(shaped.iter().all(|point| (point.energy - 40.0).abs() < 1e-9));
        // A half hour with a point without a power reading is left as it is.
        let unshaped = shaped_by_power(vec![point(DAY, 39.0, 14000.0), TelemetryPoint { demand: None, ..point(DAY + 10, 43.0, 0.0) }]);
        assert_eq!(unshaped.iter().map(|point| point.energy).collect::<Vec<_>>(), [39.0, 43.0]);
    }
}
