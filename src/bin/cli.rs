use marco_sparko::Cli;

#[tokio::main]
async fn main() {

    match Cli::new().await {
        Ok(ms) => {
            let mut cli = ms;

            if let Err(error) = cli.run().await {
                println!("Execution failed: {}", error);
            }
        },
        Err(error) => println!("Initialization failed: {}", error),
    }
}