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

use std::sync::Arc;

use graphql_client::GraphQLQuery;

use crate::Error;


// #[derive(Debug)]
pub struct RequestManager {
    reqwest_client: reqwest::Client,
    url: String,
}

impl RequestManager {
    pub fn new(url: String) -> Result<RequestManager, Box<dyn std::error::Error>> {
        Ok(RequestManager {
            reqwest_client: reqwest::Client::builder()
                .user_agent("marco_sparko/0.0.1")
                .default_headers(
                    std::iter::once((
                        reqwest::header::CONTENT_TYPE,
                        reqwest::header::HeaderValue::from_str("application/json")?
                    ))
                    .collect(),
                )
                .build()?,
            url,
        })
    }


    pub async fn call<Q: GraphQLQuery>(&self, variables: <Q as GraphQLQuery>::Variables) 
    -> Result<Q::ResponseData, Box<dyn std::error::Error>> {
        self.do_call::<Q>(variables, None).await
    }

    pub async fn do_call<Q: GraphQLQuery>(&self, variables: Q::Variables, token: Option<&Arc<String>>) 
    -> Result<Q::ResponseData, Box<dyn std::error::Error>> {
        let request_body = Q::build_query(variables);

               
        // let client = reqwest::Client::builder()
        // .user_agent("graphql-rust/0.10.0")
        // .default_headers(
        //     std::iter::once((
        //         reqwest::header::AUTHORIZATION,
        //         reqwest::header::HeaderValue::from_str(&format!("Bearer {}", token))
        //             .unwrap(),
        //     ))
        //     .collect(),
        // )
        // .build()?;


    // let serialized = serde_json::to_string(&request_body).unwrap();

    let mut request = self.reqwest_client.post(&self.url.clone());

    if let Some(token) = token {
        request = request.header(reqwest::header::AUTHORIZATION, reqwest::header::HeaderValue::from_str(token)?);
    }

    let res = request
        .json(&request_body)
        // .body(serialized)
        .send().await?;
    let response_body: graphql_client::Response<Q::ResponseData> = res.json().await?;

    if let Some(errors) = response_body.errors {
        Err(Box::new(Error::GraphQLError(errors)))
    }
    else {
        if let Some(data) = response_body.data {
            Ok(data)
        }
        else {
            Err(Box::new(Error::InternalError(format!("No result found"))))
        }
    }

    // Ok(response_body)
    // println!("{:#?}", response_body);

    // let response_body =
    //     post_graphql::<getAccountPropertiesMeters, _>(&client, "https://api.octopus.energy/v1/graphql", variables).await.unwrap();

    //     if let Some(errors) = response_body.errors {
    //         eprintln!("there are errors:");
    
    //         for error in &errors {
    //             eprintln!("{:?}", error);
    //         }
    //     }
    
    // if let Some(response_data) = response_body.data {
    //     Ok(response_data)
    // }
    // else {
    //     Err(Box::new(Error::InternalError(format!("No response data"))))
    // }


    }
}