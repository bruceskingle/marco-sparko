//! The latest numbers, below the clock, see main.rs, of which each feature which captures sets its own lines, e.g. the
//! power of each of the gateway's meters, see Panel::set, or, until any does, e.g. as none is set up yet, where to find
//! the device, so that it can be adopted, see Screen::unconfigured.

use std::collections::BTreeMap;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::{Arc, Mutex};

use embedded_graphics::mono_font::MonoTextStyle;
use embedded_graphics::mono_font::ascii::FONT_10X20;
use embedded_graphics::prelude::*;
use embedded_graphics::primitives::{PrimitiveStyle, Rectangle};
use embedded_graphics::text::{Baseline, Text};
use sparko_embedded_std::graphics::{Color, DisplayManager, DrawTargetResultExt};
use sparko_embedded_std::platform::PlatformInitializer;
use sparko_embedded_std::task::scheduler::ScheduledTask;
use sparko_esp_idf::{Esp32Platform, Esp32PlatformInitializer};

use crate::device::captured_by;

use crate::CLOCK_HEIGHT;

/// The height and width of a character of FONT_10X20, which fits 24 across the CYD's 240 pixels, and 32 across the
/// Waveshare's 320.
const LINE_HEIGHT: i32 = 20;
const CHARACTER_WIDTH: u32 = 10;
/// The longest a line is kept, the most which fit across either.
const LINE_LENGTH: usize = 32;

/// A line, and whether it is a warning, shown in yellow.
pub type Line = (String, bool);

/// What the screen shows below the clock: each feature's lines, in the order of the number each sets them with.
pub struct Panel {
    sections: Mutex<BTreeMap<u8, Vec<Line>>>,
    /// Changed whenever the lines are, so the screen only redraws when it must.
    generation: AtomicU32,
    started: AtomicBool,
}

impl Panel {
    pub fn new() -> Arc<Panel> {
        Arc::new(Panel { sections: Mutex::new(BTreeMap::new()), generation: AtomicU32::new(0), started: AtomicBool::new(false) })
    }

    /// Sets a feature's lines, cut to the width of the screen, order being where they go.
    pub fn set(&self, order: u8, mut lines: Vec<Line>) {
        for (line, _) in &mut lines {
            if let Some((index, _)) = line.char_indices().nth(LINE_LENGTH) {
                line.truncate(index);
            }
        }
        self.sections.lock().unwrap().insert(order, lines);
        self.generation.fetch_add(1, Ordering::SeqCst);
    }

    /// Starts drawing it, the first time a feature asks, from its init, which is whether or not it is enabled, so that
    /// a device with none enabled shows where to find it.
    pub fn start(self: &Arc<Self>, initializer: &mut Esp32PlatformInitializer) -> anyhow::Result<()> {
        if !self.started.swap(true, Ordering::SeqCst) {
            initializer.add_task(Box::new(Screen { panel: self.clone(), shown: None }), "*/2 * * * * *")?;
        }
        Ok(())
    }
}

struct Screen {
    panel: Arc<Panel>,
    /// The panel's and display's generations, and whether the device was adopted, last shown, so that it is only
    /// redrawn when any changes, e.g. after adoption's pairing code has been shown, see DisplayManager::generation.
    shown: Option<(u32, u32, bool)>,
}

impl Screen {
    /// Where to find the device, and what it needs, shown while no feature has set any lines, e.g. as none is
    /// enabled, as it has not been set up yet.
    fn unconfigured(sparko: &Esp32Platform, adopted: bool) -> Vec<Line> {
        let address = sparko.ip_address().map_or_else(|| "Not on the Wi-Fi".to_string(), |address| address.to_string());
        let needs = if adopted { "Nothing set up yet" } else { "Not adopted yet" };
        vec![
            (captured_by(sparko), false),
            (address, false),
            (needs.to_string(), true),
            ("Find it on the app's".to_string(), false),
            ("Devices page".to_string(), false),
        ]
    }
}

impl ScheduledTask<Esp32Platform> for Screen {
    fn run(&mut self, sparko: &mut Esp32Platform) -> anyhow::Result<()> {
        let generation = self.panel.generation.load(Ordering::SeqCst);
        let adopted = sparko.is_adopted();
        let display_manager = sparko.display_manager.clone();
        let mut manager = display_manager.lock().unwrap();
        if manager.overlay_active() {
            return Ok(());
        }
        let shown = (generation, manager.generation(), adopted);
        if self.shown == Some(shown) {
            return Ok(());
        }
        self.shown = Some(shown);
        let mut lines: Vec<Line> = self.panel.sections.lock().unwrap().values().flatten().cloned().collect();
        if lines.is_empty() {
            lines = Self::unconfigured(sparko, adopted);
        }

        let background = manager.map_color(&Color::Black);
        let text = MonoTextStyle::new(&FONT_10X20, manager.map_color(&Color::White));
        let warning = MonoTextStyle::new(&FONT_10X20, manager.map_color(&Color::Yellow));
        let display = manager.display();
        let bounding_box = display.bounding_box();
        let area = Rectangle::new(
            Point::new(bounding_box.top_left.x, bounding_box.top_left.y + CLOCK_HEIGHT as i32),
            Size::new(bounding_box.size.width, bounding_box.size.height.saturating_sub(CLOCK_HEIGHT)),
        );
        area.into_styled(PrimitiveStyle::with_fill(background)).draw(display).anyhow()?;
        let max_lines = (area.size.height as i32 / LINE_HEIGHT) as usize;
        let fit = (area.size.width / CHARACTER_WIDTH) as usize;
        for (index, (line, is_warning)) in lines.iter().take(max_lines).enumerate() {
            let position = area.top_left + Point::new(0, index as i32 * LINE_HEIGHT);
            let style = if *is_warning { warning } else { text };
            let line = line.char_indices().nth(fit).map_or(line.as_str(), |(end, _)| &line[..end]);
            Text::with_baseline(line, position, style, Baseline::Top).draw(display).anyhow()?;
        }
        Ok(())
    }

    fn name(&self) -> &str {
        "Panel"
    }
}
