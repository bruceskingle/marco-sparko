use graphql_client::{GraphQLQuery, Response};
use std::error::Error;
use reqwest;

type GenericScalar = String;

// #[derive(GraphQLQuery)]
// #[graphql(
//     schema_path = "src/schema.graphql",
//     query_path = "src/getAccount.graphql",
//     response_derives = "Debug",
// )]
// pub struct ObtainJSONWebTokenInput;

// #[derive(GraphQLQuery)]
// #[graphql(
//     schema_path = "src/schema.graphql",
//     query_path = "src/getAccount.graphql",
//     response_derives = "Debug",
// )]
// pub struct GetAccount;

#[allow(clippy::upper_case_acronyms)]
#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "graphql/octopus/octopus-schema.graphql",
    query_path = "graphql/octopus/getAccountPropertiesMeters.graphql",
    response_derives = "Debug",
)]
pub struct GetAccountPropertiesMeters;



// pub struct obtainKrakenToken;
// pub struct ObtainJSONWebTokenInput;

// type URI = String;

// #[derive(GraphQLQuery)]
// #[graphql(
//     schema_path = "src/git_schema.graphql",
//     query_path = "src/git_query.graphql",
//     response_derives = "Debug"
// )]
// struct RepoView;

// async fn perform_my_query(variables: union_query::Variables) -> Result<(), Box<dyn Error>> {

//     // this is the important line
//     let request_body = UnionQuery::build_query(variables);

//     let client = reqwest::Client::new();
//     let mut res = client.post("/graphql").json(&request_body).send().await?;
//     let response_body: Response<union_query::ResponseData> = res.json().await?;
//     println!("{:#?}", response_body);
//     Ok(())
// }

// fn main() {
//     println!("Hello world");

//     let API_KEY = std::env::var("OCTOPUS_API_KEY").expect("Missing OCTOPUS_API_KEY env var");

//     let variables = union_query::Variables {
//         APIKey: API_KEY
//     };
// }