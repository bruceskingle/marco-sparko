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
        .with_schema("graphql/octopus/octopus-schema.graphql")
        .with_query("graphql/octopus/Login.graphql", "login")
        .build()?;

    // panic!("Panic test!"); 
    Ok(())
}
