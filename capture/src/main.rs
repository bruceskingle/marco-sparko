//! A Capture Device: an ESP32 which polls what Marco Sparko needs while the computer it runs on sleeps, keeps it, and
//! gives it to the app, which reads it from the device's /capture/... pages. For now only the Tesla gateway, see
//! tesla.rs, on a CYD, whose screen shows the clock with the latest numbers below it.

mod records;
mod screen;
mod tesla;

use embedded_graphics::prelude::{Point, Size};
use embedded_graphics::primitives::Rectangle;
use sparko_embedded_std::DisplayOrientation;
use sparko_esp_idf::Esp32Platform;
use sparko_esp_idf::features::analog_clock::AnalogClock;

/// The clock takes the top of the screen, and the numbers the rest, see screen.rs.
pub const CLOCK_HEIGHT: u32 = 200;

fn main() -> anyhow::Result<()> {
    let (builder, _remainder) = Esp32Platform::builder()?;

    let sparko_esp32 = builder
        .with_feature(Box::new(
            AnalogClock::builder()
                .with_layout(|bounding_box: &Rectangle| {
                    Rectangle::new(
                        Point::new(bounding_box.top_left.x + 1, bounding_box.top_left.y + 1),
                        Size::new(bounding_box.size.width - 2, CLOCK_HEIGHT - 2),
                    )
                })
                .build()?,
        ))?
        .with_feature(Box::new(tesla::TeslaCapture::new()))?
        .with_display_orientation(DisplayOrientation::Rotate0)?
        .build()?;

    sparko_esp32.start()
}
