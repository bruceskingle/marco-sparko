use marco_sparko::MarcoSparko;

#[tokio::main]
async fn main() {
    match MarcoSparko::new().await {
        Ok(ms) => {
            let mut marco_sparko = ms; //Box::new(ms);

            if let Err(error) = marco_sparko.run().await {
                println!("Execution failed: {}", error);
            }
        },
        Err(error) => println!("Initialization failed: {}", error),
    }
}