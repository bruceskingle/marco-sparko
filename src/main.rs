use marco_sparko::{ Args, Cli, components::app::App};

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

    if args.marco_sparko_args.cli {
        cli_main(args);
    }
    else {
        #[cfg(feature = "desktop")]
        fn launch_app() {
            let window = dioxus::desktop::tao::window::WindowBuilder::new()
                .with_resizable(true);

            dioxus::LaunchBuilder::new()
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