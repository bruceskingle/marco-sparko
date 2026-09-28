use marco_sparko::components::app::App;

fn main() {
    #[cfg(feature = "desktop")]
    fn launch_app() {
        let window = dioxus::desktop::tao::window::WindowBuilder::new()
            .with_resizable(true);

        dioxus::LaunchBuilder::new()
            .with_cfg(dioxus::desktop::Config::new()
                .with_window(window)
                // .with_menu(None)
            )
            .launch(App);
    }

    #[cfg(not(feature = "desktop"))]
    fn launch_app() {
        dioxus::launch(App);
    }

    launch_app();
}