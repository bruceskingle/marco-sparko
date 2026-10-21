use marco_sparko::components::app::App;

fn main() {
    #[cfg(feature = "desktop")]
    fn launch_app() {
        // The size is the default for the first run, after that the size from the last run is used.
        let window = dioxus::desktop::tao::window::WindowBuilder::new()
            .with_title("Marco Sparko")
            .with_inner_size(dioxus::desktop::LogicalSize::new(1400, 768))
            .with_resizable(true);

        dioxus::LaunchBuilder::new()
            .with_cfg(marco_sparko::window_state::remember_window_state(
                dioxus::desktop::Config::new()
                    // .with_menu(None)
                    ,
                window,
            ))
            .launch(App);
    }

    #[cfg(not(feature = "desktop"))]
    fn launch_app() {
        dioxus::launch(App);
    }

    launch_app();
}