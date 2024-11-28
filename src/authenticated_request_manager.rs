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
use sparko_graphql::TokenManager;

use crate::{RequestManager};


// #[derive(Debug)]
pub struct AuthenticatedRequestManager<T: TokenManager> {
    request_manager: Arc<RequestManager>,
    token_manager: T,
}

impl<T: TokenManager> AuthenticatedRequestManager<T> {
    pub fn new(request_manager: Arc<RequestManager>, token_manager: T) -> Result<AuthenticatedRequestManager<T>, Box<dyn std::error::Error>> {
        Ok(AuthenticatedRequestManager {
            request_manager,
            token_manager,
        })
    }


    pub async fn call<Q: GraphQLQuery>(&mut self, variables: Q::Variables) 
    -> Result<<Q as GraphQLQuery>::ResponseData, Box<dyn std::error::Error>> {
        let token = &self.token_manager.get_authenticator().await?;

        eprintln!("AuthenticatedRequestManager token=<{}>", token);
        let result = self.request_manager.do_call::<Q>(variables, Some(token)).await;

        if let Err(e) = &result {
            eprintln!("Result {:?}", e);
        }

        // if let Ok(v) = &result {
        //     eprintln!("Result {:?}", v);
        // }
        
        result

        // if let Some(data) = response.data {
        //     Ok(data)
        // }
        // else {
        //     if let Some(errors) = response.errors {
        //         Err(Box::new(Error::UserError(serde_json::to_string(&errors)?)))
        //     }
        //     else {
        //         Err(Box::new(Error::InternalError(format!("No result found"))))
        //     }
        // }
    }
}