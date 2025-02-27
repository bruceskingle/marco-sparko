use std::{error::Error, process};

// fn get_current_working_dir() -> String {
//     let res = std::env::current_dir();
//     match res {
//         Ok(path) => path.into_os_string().into_string().unwrap(),
//         Err(_) => "FAILED".to_string()
//     }
// }
fn main() {
    if let Err(_) = build() {
        process::exit(1);
        // panic!("Build failed {}", error);
    }
}

// Example custom build script.
fn build()  -> Result<(), Box<dyn Error>> {
    // panic!("Panic cwd={}", get_current_working_dir()); 

    println!("cargo::rerun-if-changed=build.rs");
    sparko_graphql_builder::builder("graphql")
        .with_type("Date", "sparko_graphql::types::Date")
        .with_type("DateTime", "sparko_graphql::types::DateTime")
        .with_type("Decimal", "crate::octopus::decimal::Decimal")
        .with_schema("graphql/octopus/octopus-schema.graphql")
        .with_query("graphql/octopus/Login.graphql", "login")
        .with_query("graphql/octopus/account.graphql", "account")
        .with_query("graphql/octopus/meter.graphql", "meter")
        .with_query("graphql/octopus/bill.graphql", "bill")
        // .with_print(true)
        .build()?;

    // panic!("Panic test!"); 
    Ok(())
}
