use marco_sparko::components::app::App;

fn main() {
    // With --fetch the latest data is fetched without a window, e.g. from a scheduled task, then the app exits.
    let args = marco_sparko::Args::ms_parse();
    if args.fetch() {
        std::process::exit(if marco_sparko::fetch_latest_data(args) { 0 } else { 1 });
    }

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
                    // The app's own menu bar, whose Window menu lists the windows.
                    .with_menu(marco_sparko::components::child_windows::menu_bar())
                    // .with_menu(None)
                    // The webview's context menu (Reload, Inspect Element...) is only offered in debug builds, its
                    // Reload would restart the app in a broken state.
                    .with_disable_context_menu(!cfg!(debug_assertions)),
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