use std::{env, error::Error, fs::File, path::Path, process};
use std::io::Write;

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
    
    if let Some(dir) = env::var_os("OUT_DIR") {
        let dest_path = Path::new(&dir).join("crate_info.rs");
        // let dest_path_string = format!("{}", dest_path.to_string_lossy());

        let mut file = File::create(dest_path)?;

        writeln!(file, r#"
mod CrateInfo {{
    pub const PACKAGE_NAME: &'static str = "{}";
    pub const PACKAGE_VERSION: &'static str = "{}";
    pub const USER_AGENT: &'static str = "{}-{}";
}}
"#, env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"), env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"))?;
    }

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
