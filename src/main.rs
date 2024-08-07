/*****************************************************************************
 MIT License

Copyright (c) 2024 Bruce Skingle

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
******************************************************************************/

use marco_sparko::octopus::{error::Error, Client};




#[tokio::main]
async fn main() -> Result<(), Error> {
    let api_key = std::env::var("OCTOPUS_API_KEY").expect("Missing OCTOPUS_API_KEY env var");

    let mut octopus_client = Client::builder()
        .with_url(String::from("https://api.octopus.energy/v1/graphql/"))
        .with_api_key(String::from(api_key))
        .build()?;



        let result = octopus_client.get_account().await;


        match result {
            Ok(account) => println!("That was OK {:?}", account),
            Err(e) => println!("\n==============================================\n\nError {:?}\n\n==============================================\n", e),
            
        }

        let _ = octopus_client.get_account().await;
    
    Ok(())
}