use marco_sparko::{ components::app::App};


fn main() -> anyhow::Result<()> {
    dioxus::launch(App);
    Ok(())
}


// use dioxus_desktop::{Config, WindowBuilder};
// use wry::dpi::LogicalSize; // <-- directly from wry

// fn main() -> anyhow::Result<()> {
//     // configure the window
//     let window = WindowBuilder::new()
//         .with_title("My Dioxus App")
//         .with_inner_size(LogicalSize::new(1024.0, 768.0)); // initial width x height

//     // launch the app with custom window config
//     dioxus_desktop::launch_cfg(App, Config::default().with_window(window));

//     Ok(())
// }