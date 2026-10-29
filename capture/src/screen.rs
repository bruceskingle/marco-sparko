//! The latest numbers, below the clock, see main.rs: the power of each of the gateway's meters, the battery's charge
//! and backup reserve, and when the gateway was last polled, or why that failed.

use std::sync::{Arc, Mutex};

use chrono::Local;
use embedded_graphics::mono_font::MonoTextStyle;
use embedded_graphics::mono_font::ascii::FONT_10X20;
use embedded_graphics::prelude::*;
use embedded_graphics::primitives::{PrimitiveStyle, Rectangle};
use embedded_graphics::text::{Baseline, Text};
use sparko_embedded_std::graphics::{Color, DisplayManager, DrawTargetResultExt};
use sparko_embedded_std::task::scheduler::ScheduledTask;
use sparko_esp_idf::Esp32Platform;

use crate::CLOCK_HEIGHT;
use crate::records::{Response, Store};

/// The height of a line of FONT_10X20, which fits 24 characters across the CYD's 240 pixels.
const LINE_HEIGHT: i32 = 20;
const LINE_LENGTH: usize = 24;

/// The meters, as named by the gateway, and as shown.
const METERS: &[(&str, &str)] = &[("site", "Grid"), ("solar", "Solar"), ("battery", "Battery"), ("load", "House")];

pub struct Screen<DM: DisplayManager> {
    display_manager: Arc<Mutex<DM>>,
    store: Arc<Mutex<Store>>,
    area: Rectangle,
    /// The store's and display's generations last shown, so that it is only redrawn when either changes, e.g. after
    /// adoption's pairing code has been shown, see DisplayManager::generation.
    shown: Option<(u32, u32)>,
}

impl<DM: DisplayManager> Screen<DM> {
    pub fn new(display_manager: &Arc<Mutex<DM>>, store: Arc<Mutex<Store>>) -> anyhow::Result<Self> {
        let bounding_box = display_manager.lock().unwrap().display().bounding_box();
        let area = Rectangle::new(
            Point::new(bounding_box.top_left.x, bounding_box.top_left.y + CLOCK_HEIGHT as i32),
            Size::new(bounding_box.size.width, bounding_box.size.height.saturating_sub(CLOCK_HEIGHT)),
        );
        Ok(Self { display_manager: display_manager.clone(), store, area, shown: None })
    }
}

/// The lines to show, each with whether it is a warning.
fn lines(store: &Store) -> Vec<(String, bool)> {
    let mut lines = Vec::new();
    if let Some(record) = store.latest() {
        for (name, label) in METERS {
            if let Some(meter) = record.meter(name) {
                lines.push((format!("{label:<8}{:>7.2} kW", meter.instant_power / 1000.0), false));
            }
        }
        // As the gateway says them, which the Tesla app shows less the 5% it keeps in reserve.
        let charge = match record.response("system_status/soe") {
            Some(Response::Soe { percentage: Some(percentage) }) => format!("{percentage:.0}%"),
            _ => "?".to_string(),
        };
        let reserve = match record.response("operation") {
            Some(Response::Operation { backup_reserve_percent: Some(reserve), .. }) => format!("{reserve:.0}%"),
            _ => "?".to_string(),
        };
        lines.push((format!("Charge {charge}  min {reserve}"), false));
    }
    match store.last_poll() {
        None => lines.push(("Waiting for first poll".to_string(), false)),
        Some(poll) => {
            let time = poll.time.with_timezone(&Local).format("%H:%M");
            match (&poll.error, store.storage_problem()) {
                (None, None) => lines.push((format!("Polled {time}"), false)),
                // e.g. no SD card, so the records will be lost when the device restarts.
                (None, Some(_)) => lines.push((format!("Polled {time}, no SD"), true)),
                (Some(error), _) => lines.push((format!("{time} {error}"), true)),
            }
        }
    }
    for (line, _) in &mut lines {
        if let Some((index, _)) = line.char_indices().nth(LINE_LENGTH) {
            line.truncate(index);
        }
    }
    lines
}

impl<DM: DisplayManager> ScheduledTask<Esp32Platform> for Screen<DM> {
    fn run(&mut self, _sparko: &mut Esp32Platform) -> anyhow::Result<()> {
        let (generation, lines) = {
            let store = self.store.lock().unwrap();
            (store.generation(), lines(&store))
        };
        let mut manager = self.display_manager.lock().unwrap();
        if manager.overlay_active() {
            return Ok(());
        }
        let shown = (generation, manager.generation());
        if self.shown == Some(shown) {
            return Ok(());
        }
        self.shown = Some(shown);

        let background = manager.map_color(&Color::Black);
        let text = MonoTextStyle::new(&FONT_10X20, manager.map_color(&Color::White));
        let warning = MonoTextStyle::new(&FONT_10X20, manager.map_color(&Color::Yellow));
        let display = manager.display();
        self.area.into_styled(PrimitiveStyle::with_fill(background)).draw(display).anyhow()?;
        let max_lines = (self.area.size.height as i32 / LINE_HEIGHT) as usize;
        // The status line is last, so it is the one kept if there are too many.
        let skip = lines.len().saturating_sub(max_lines);
        for (index, (line, is_warning)) in lines.iter().skip(skip).enumerate() {
            let position = self.area.top_left + Point::new(0, index as i32 * LINE_HEIGHT);
            let style = if *is_warning { warning } else { text };
            Text::with_baseline(line, position, style, Baseline::Top).draw(display).anyhow()?;
        }
        Ok(())
    }

    fn name(&self) -> &str {
        "Tesla Screen"
    }
}
