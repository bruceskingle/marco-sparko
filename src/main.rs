use clap::Parser;
use marco_sparko::{ Args, Cli, components::app::App};


// fn main() -> anyhow::Result<()> {
//     dioxus::launch(App);
//     Ok(())
// }

#[tokio::main]
async fn cli_main(args: Args) {

    match Cli::new(args).await {
        Ok(ms) => {
            let mut cli = ms;

            if let Err(error) = cli.run().await {
                println!("Execution failed: {}", error);
            }
        },
        Err(error) => println!("Initialization failed: {}", error),
    }
}

fn main() {
    let args = Args::ms_parse();

    println!("Args: {:?}", args.marco_sparko_args);
    println!("Module Args: {:?}", args.module_args);
    if args.marco_sparko_args.cli {
        cli_main(args);
    }
    else {
        #[cfg(feature = "desktop")]
        fn launch_app() {
            let window = dioxus::desktop::tao::window::WindowBuilder::new()
                .with_resizable(true);

            dioxus::LaunchBuilder::new()
                // .with_context(args)
                .with_cfg(dioxus::desktop::Config::new()
                    .with_window(window)
                    .with_menu(None))
                .launch(App);
        }

        #[cfg(not(feature = "desktop"))]
        fn launch_app() {
            dioxus::launch(App);
        }

        launch_app();
    }
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