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

use reqwest::StatusCode;
use serde::{Deserialize, Serialize};



#[derive(Debug)]
pub enum Error {
    GraphQLError(Vec<GraphQLJsonError>),
    IOError(reqwest::Error),
    JsonError(serde_json::Error),
    HttpError(StatusCode),
    NoData,
}

impl From<reqwest::Error> for Error {
    fn from(err: reqwest::Error) -> Error {
        Error::IOError(err)
    }
}


impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Error {
        Error::JsonError(err)
    }
}


#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct Location {
   line: i32,
   column: i32,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct ValidationError {
       message: String,
       input_path: Vec<String>
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct Extensions {
   error_type: String,
   error_code: String,
   error_description: String,
   error_class: String,
   validation_errors: Vec<ValidationError>
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GraphQLJsonError {
   message: Option<String>,
   locations: Vec<Location>,
   path: Vec<String>,
   extensions: Extensions,
}

// #[derive(Serialize, Deserialize, Debug)]
// struct Variable {
//     name:   String,
//     value:  dyn Serialize,
// }


// #[derive(Serialize, Deserialize, Debug)]
// struct VariableMap {
//     vars:   Vec<Variable>,
//     map:    HashMap<&str, &Variable>
// }

// impl VariableMap {
//     fn new() -> VariableMap {
//         VariableMap {
//             vars: Vec::new(),
//             map: HashMap::new(),
//         }
//     }
// }
