//! # cryptol-rust-client
//!
//! `cryptol-rust-client` is a collection of utilities for connecting
//! to and interacting with a running `cryptol-remote-api` instance.

use std::env;

use serde::{ Serialize, Deserialize };
use serde_json::json;

use jsonrpsee::core::client::ClientT;
use jsonrpsee::http_client::{ HeaderMap, HeaderValue, HttpClientBuilder, HttpClient };
use jsonrpsee::core::params::ObjectParams;

use std::time::Duration;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

/// This structure represents the JSON blob returned by cryptol-remote-api.
/// For example:
///   `{"answer":[],"state":"a4909ccf-3ef9-45cc-913b-57e58da75788","stderr":"","stdout":""}`

#[derive(Debug, Serialize, Deserialize)]
struct CryptolResult {
    #[serde(default)]
    answer: serde_json::Value,
    state:  String,
    stderr: String,
    stdout: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Answer {
    #[serde(rename = "type")]
    answer_type: serde_json::Value,
    #[serde(rename = "type string")]
    type_string: String,
    pub value: serde_json::Value,
}

/// This structure represents the JSON blob returned by
/// cryptol-remote-api on error.  However, there is currently no way
/// to access this information using the `jsonrpsee` crate.
///
/// Example JSON blob:
///   `{"code":20500,"data":{"data":{"path":["client","//.cryptol","/usr/local/share/cryptol"],"source":"Floataboat","warnings":[]},"stderr":"","stdout":""},"message":"[error] Could not find module NoModule\nSearched paths:\n    //.cryptol\n    /usr/local/share/cryptol\nSet the CRYPTOLPATH environment variable to search more directories"}`

#[derive(Serialize, Deserialize)]
pub struct CryptolError {
    code:    i64,
    data:    CryptolErrorData,
    message: String,
}

#[derive(Serialize, Deserialize)]
pub struct CryptolErrorData {
    data:   CryptolDataData,
    stderr: String,
    stdout: String,
}

#[derive(Serialize, Deserialize)]
pub struct CryptolDataData {
    path:     Vec<String>,
    source:   String,
    warnings: Vec<Option<serde_json::Value>>,
}

/// Cryptol client struct. Contains the active client connection and
/// state attribute.

#[derive(Debug, Clone)]
pub struct CryptolClient {
    client: HttpClient,
    state:  String,
    answer: serde_json::Value,
}

/// Cryptol client implementation.

impl CryptolClient {
    
    /// This function establishes an HTTP connection with
    /// cryptol-remote-api located at CRYPTOL_SERVER_URL. Upon
    /// connection, cryptol-remote-api will load the Cryptol prelude
    /// return a token representing the state of the connection.
    ///
    /// This function has asynchronous behavior due to the POST request
    /// to cryptol-remote-api. We block on the request using
    /// #[tokio::main].
    
    #[tokio::main]
    pub async fn connect() -> Result<CryptolClient> {
        // Deduce whether or not `CRYPTOL_SERVER_URL` is defined.
        let cryptol_server_url = match env::var("CRYPTOL_SERVER_URL") {
            Ok(val) => {
                println!("Attempting to connect to cryptol-remote-api at {val}.");
                val
            },
            Err(e)  => return Err(e.into()),
        };
        
        // Insert a 'keep-alive' command into the initial packet
        // header. Perhaps unnecessary?
        let mut headers = HeaderMap::new();
        headers.insert("Connection", HeaderValue::from_static("keep-alive"));
        
        // Build client
        let client = HttpClientBuilder::default()
            .set_headers(headers)
            .request_timeout(Duration::from_secs(60 * 60))  // Set longer request timeout
            .build(cryptol_server_url)?;
        
        // Create parameters for loading the Cryptol prelude.
        let mut params = ObjectParams::new();
        params.insert("state", json!(null)).unwrap();
        params.insert("module name", "Cryptol").unwrap();
        
        // Make a request to cryptol-remote-api to load the Cryptol prelude
        let response: CryptolResult = client.request("load module", params).await?;
        
        // Create and return a new CryptolClient object to represent the
        // stateful connection
        Ok(CryptolClient { client
                         , state: response.state.clone()
                         , answer: response.answer
                         })
    }

    /// This function sends requests to cryptol-remote-api in the form
    /// of a given action and parameters.
    ///
    /// This function has asynchronous behavior due to the POST request
    /// to cryptol-remote-api. We block on the request using
    /// #[tokio::main].
    ///
    /// Sample JSON for this:
    ///   `{"function": "sha384", "arguments": ["1 : [16]"], "state": "7dc51618-e655-49a3-9a72-880eeb8e16dd"}`
    ///
    ///   `{"answer":{"type":{"forall":[],"propositions":[],"type":{"type":"bitvector","width":{"type":"number","value":384}}},"type string":"[384]","value":{"data":"5d13bb39a64c4ee16e0e8d2e1c13ec4731ff1ac69652c072d0cdc355eb9e0ec41b08aef3dd6fe0541e9fa9e3dcc80f7b","encoding":"hex","expression":"bits","width":384}},"state":"fa57d2ec-afa8-4d7a-b1f2-f3b47412f13d","stderr":"","stdout":""}`
    
    #[tokio::main]
    async fn request(&mut self, action: &str, params: ObjectParams) -> Result<()> {
        // Make a request to cryptol-remote-api to load the Cryptol prelude
        let response: CryptolResult = self.client.request(action, params).await?;

        // It would be nice to parse out any failure from this
        // response.  See the `CryptolError` struct above -- Cryptol
        // does return a nice `message` with pertinent inforamtion
        // about the failure. To do this we would need to access the
        // resulting JSON blob when `request` returns `Err`. The `Err`
        // message does not contain much information.
        
        // Update the CryptolClient state.
        self.state = response.state.clone();
        
        // Update the CryptolClient answer.
        self.answer = response.answer;

        Ok(())
    }
    
    /// This function loads the given Cryptol module existing in the
    /// CRYPTOL_PATH of cryptol-remote-api.
    
    pub fn load_module(&mut self, module: &str) -> Result<()> {
        // Create parameters for loading the given Cryptol module.
        let mut params = ObjectParams::new();
        params.insert("state", json!(self.state)).unwrap();
        params.insert("module name", module).unwrap();
        
        // Make a request to cryptol-remote-api to load the Cryptol prelude
        self.request("load module", params)?;

        Ok(())
    }
    
    /// This function calls the given function in the loaded Cryptol
    /// module.

    pub fn call<P: Serialize>(&mut self, function: &str, arguments: Vec<P>) -> Result<Answer> {
        // Create parameters for loading the given Cryptol module.
        let mut params = ObjectParams::new();
        params.insert("state", json!(self.state)).unwrap();
        params.insert("function", json!(function)).unwrap();
        params.insert("arguments", json!(arguments)).unwrap();
        
        // Make a request to cryptol-remote-api to load the Cryptol prelude
        self.request("call", params)?;
        
        // Let `call` return the result as an Answer struct.
        let answer: Answer = serde_json::from_value(self.answer.clone()).unwrap();
        
        Ok(answer)
    }
}
