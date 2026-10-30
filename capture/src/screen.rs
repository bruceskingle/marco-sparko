//! The latest numbers, below the clock, see main.rs, of which each feature which captures sets its own lines, e.g. the
//! power of each of the gateway's meters, see Panel::set.

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

use crate::CLOCK_HEIGHT;

/// The height of a line of FONT_10X20, which fits 24 characters across the CYD's 240 pixels.
const LINE_HEIGHT: i32 = 20;
const LINE_LENGTH: usize = 24;

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

    /// Starts drawing it, the first time a feature asks, as any of them may be the one which is enabled.
    pub fn start(self: &Arc<Self>, sparko: &Esp32Platform, initializer: &mut Esp32PlatformInitializer) -> anyhow::Result<()> {
        if !self.started.swap(true, Ordering::SeqCst) {
            initializer.add_task(Box::new(Screen::new(&sparko.display_manager, self.clone())?), "*/2 * * * * *")?;
        }
        Ok(())
    }
}

struct Screen<DM: DisplayManager> {
    display_manager: Arc<Mutex<DM>>,
    panel: Arc<Panel>,
    area: Rectangle,
    /// The panel's and display's generations last shown, so that it is only redrawn when either changes, e.g. after
    /// adoption's pairing code has been shown, see DisplayManager::generation.
    shown: Option<(u32, u32)>,
}

impl<DM: DisplayManager> Screen<DM> {
    fn new(display_manager: &Arc<Mutex<DM>>, panel: Arc<Panel>) -> anyhow::Result<Self> {
        let bounding_box = display_manager.lock().unwrap().display().bounding_box();
        let area = Rectangle::new(
            Point::new(bounding_box.top_left.x, bounding_box.top_left.y + CLOCK_HEIGHT as i32),
            Size::new(bounding_box.size.width, bounding_box.size.height.saturating_sub(CLOCK_HEIGHT)),
        );
        Ok(Self { display_manager: display_manager.clone(), panel, area, shown: None })
    }
}

impl<DM: DisplayManager> ScheduledTask<Esp32Platform> for Screen<DM> {
    fn run(&mut self, _sparko: &mut Esp32Platform) -> anyhow::Result<()> {
        let generation = self.panel.generation.load(Ordering::SeqCst);
        let mut manager = self.display_manager.lock().unwrap();
        if manager.overlay_active() {
            return Ok(());
        }
        let shown = (generation, manager.generation());
        if self.shown == Some(shown) {
            return Ok(());
        }
        self.shown = Some(shown);
        let lines: Vec<Line> = self.panel.sections.lock().unwrap().values().flatten().cloned().collect();

        let background = manager.map_color(&Color::Black);
        let text = MonoTextStyle::new(&FONT_10X20, manager.map_color(&Color::White));
        let warning = MonoTextStyle::new(&FONT_10X20, manager.map_color(&Color::Yellow));
        let display = manager.display();
        self.area.into_styled(PrimitiveStyle::with_fill(background)).draw(display).anyhow()?;
        let max_lines = (self.area.size.height as i32 / LINE_HEIGHT) as usize;
        for (index, (line, is_warning)) in lines.iter().take(max_lines).enumerate() {
            let position = self.area.top_left + Point::new(0, index as i32 * LINE_HEIGHT);
            let style = if *is_warning { warning } else { text };
            Text::with_baseline(line, position, style, Baseline::Top).draw(display).anyhow()?;
        }
        Ok(())
    }

    fn name(&self) -> &str {
        "Panel"
    }
}
