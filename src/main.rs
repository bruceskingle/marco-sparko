use marco_sparko::{Error, MarcoSparko};

#[tokio::main]
async fn main() -> Result<(), Error> {
    let mut marco_sparko = MarcoSparko::new_cli()?;

    if let Err(error) = marco_sparko.run().await {
        println!("Execution failed: {}", error);
    }
    
    Ok(())
}