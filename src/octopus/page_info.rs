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

use display_json::DisplayAsJsonPretty;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, DisplayAsJsonPretty)]
#[serde(rename_all = "camelCase")]
pub struct ForwardPageInfo {
    pub start_cursor: String,
    pub has_next_page: bool
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ForwardPageOf<T> 
{
    pub page_info: ForwardPageInfo,
    pub edges: Vec<EdgeOf<T>>
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct  EdgeOf<T>
{
  pub node: T
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_forward() {
        let json = r#"
{
  "startCursor": "YXJyYXljb25uZWN0aW9uOjA=",
  "hasNextPage": true
}
        "#;

        let value = serde_json::from_str(json).unwrap();
        let forward_page_info = ForwardPageInfo::from(value);

        assert_eq!(forward_page_info.start_cursor, "YXJyYXljb25uZWN0aW9uOjA=");
        assert_eq!(forward_page_info.has_next_page, true);
    }
}