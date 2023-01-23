use std::env;

use serde::{Serialize, Deserialize};
use serde_json::{json, from_str, Value};

use jsonrpsee::core::client::ClientT;
use jsonrpsee::http_client::{ HeaderMap, HeaderValue, HttpClientBuilder, HttpClient };
use jsonrpsee::rpc_params;
use jsonrpsee::core::params::ObjectParams;

use std::time::Duration;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

/**
 * This structure represents the JSON blob returned by cryptol-remote-api.
 * For example:
 *   {"answer":[],"state":"a4909ccf-3ef9-45cc-913b-57e58da75788","stderr":"","stdout":""}
 */

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
    value: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SHA384ResultValue {
    data: String,
    encoding: String,
    expression: String,
    #[serde(default)]
    width: Option<i64>,
}

/**
 * This structure represents the JSON blob returned by cryptol-remote-api on error.
 * For example:
 * {"code":20500,"data":{"data":{"path":["client","//.cryptol","/usr/local/share/cryptol"],"source":"Floataboat","warnings":[]},"stderr":"","stdout":""},"message":"[error] Could not find module Floataboat\nSearched paths:\n    //.cryptol\n    /usr/local/share/cryptol\nSet the CRYPTOLPATH environment variable to search more directories"}
 */

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

/**
 * Cryptol client struct. Contains the active client connection and
 * state attribute.
 */

#[derive(Debug, Clone)]
struct CryptolClient {
  client: HttpClient,
  state:  String,
  answer: serde_json::Value,
}

/**
 * Cryptol client implementation.
 */

impl CryptolClient {

  /**
   * This function establishes an HTTP connection with
   * cryptol-remote-api located at CRYPTOL_SERVER_URL. Upon connection,
   * cryptol-remote-api will load the Cryptol prelude return a token
   * representing the state of the connection.
   *
   * This function has asynchronous behavior due to the POST request to
   * cryptol-remote-api. We block on the request using #[tokio::main].
   */

  #[tokio::main]
  async fn connect() -> Result<CryptolClient> {
    // Deduce whether or not `CRYPTOL_SERVER_URL` is defined.
    let cryptol_server_url = match env::var("CRYPTOL_SERVER_URL") {
      Ok(val) => {
        println!("Attempting to connect to cryptol-remote-api at {}.", val);
        val
      },
      Err(e) => return Err(e.into()),
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
    Ok(CryptolClient { client: client
                     , state: response.state.clone()
                     , answer: response.answer.clone() })
  }

  /**
   * This function loads the given Cryptol module existing in the
   * CRYPTOL_PATH of cryptol-remote-api.
   *
   * This function has asynchronous behavior due to the POST request to
   * cryptol-remote-api. We block on the request using #[tokio::main].
   */

  #[tokio::main]
  async fn load_module(&mut self, module: &str) -> Result<()> {
    // Create parameters for loading the given Cryptol module.
    let mut params = ObjectParams::new();
    params.insert("state", json!(self.state)).unwrap();
    params.insert("module name", module).unwrap();

    // Make a request to cryptol-remote-api to load the Cryptol prelude
    let response: CryptolResult = self.client.request("load module", params).await?;

    /* It would be nice to parse out any failure from this response.
     * See the `CryptolError` struct above -- Cryptol does return a
     * nice `message` with pertinent inforamtion about the failure.
     * Right now I'm not sure how to access the JSON blob when
     * `request` returns Err, and the actual Err message does not
     * contain much information.
     */

    // Update the CryptolClient state.
    self.state  = response.state.clone();
    self.answer = response.answer.clone();

    Ok(())
  }

  /**
   * This function calls the given function in the loaded Cryptol module.
   *
   * This function has asynchronous behavior due to the POST request to
   * cryptol-remote-api. We block on the request using #[tokio::main].
   *
   * Sample JSON for this:
   *   {"function": "sha384", "arguments": ["1 : [16]"], "state": "7dc51618-e655-49a3-9a72-880eeb8e16dd"}
   *
   *   {"answer":{"type":{"forall":[],"propositions":[],"type":{"type":"bitvector","width":{"type":"number","value":384}}},"type string":"[384]","value":{"data":"5d13bb39a64c4ee16e0e8d2e1c13ec4731ff1ac69652c072d0cdc355eb9e0ec41b08aef3dd6fe0541e9fa9e3dcc80f7b","encoding":"hex","expression":"bits","width":384}},"state":"fa57d2ec-afa8-4d7a-b1f2-f3b47412f13d","stderr":"","stdout":""}
   */

  #[tokio::main]
  async fn call<P: Serialize>(&mut self, function: &str, arguments: Vec<P>) -> Result<()> {
    // Create parameters for loading the given Cryptol module.
    let mut params = ObjectParams::new();
    params.insert("state", json!(self.state)).unwrap();
    params.insert("function", json!(function)).unwrap();
    params.insert("arguments", json!(arguments)).unwrap();

    // Make a request to cryptol-remote-api to load the Cryptol prelude
    let response: CryptolResult = self.client.request("call", params).await?;

    /* It would be nice to parse out any failure from this response.
     * See the `CryptolError` struct above -- Cryptol does return a
     * nice `message` with pertinent inforamtion about the failure.
     * Right now I'm not sure how to access the JSON blob when
     * `request` returns Err, and the actual Err message does not
     * contain much information.
     */

    // Update the CryptolClient state.
    self.state = response.state.clone();

    // Update the CryptolClient answer.
    self.answer = serde_json::from_value(response.answer).unwrap();

    println!("{:?}", self);

    Ok(())
  }

}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_connect() {
    let cryptol_client = CryptolClient::connect();
    assert!(cryptol_client.is_ok());
    println!("{:?}", cryptol_client);
  }

  #[test]
  fn test_load_module_success() {
    let mut cryptol_client = match CryptolClient::connect() {
      Ok(c) => c,
      Err(e) => panic!("An error occurred while connection to cryptol-remote-api: {}", e),
    };

    match cryptol_client.load_module("SuiteB") {
      Ok(()) => (),
      Err(e) => panic!("Loading module failed: {}", e),
    };
  }

  #[test]
  fn test_load_module_failure() {
    let mut cryptol_client = match CryptolClient::connect() {
      Ok(c) => c,
      Err(e) => panic!("An error occurred while connection to cryptol-remote-api: {}", e),
    };

    match cryptol_client.load_module("nosuchmodule") {
      Ok(()) => panic!("nosuchmodule should not exist"),
      Err(e) => (),
    };
  }

  #[test]
  fn test_call_sha384_success() {
    let mut cryptol_client = match CryptolClient::connect() {
      Ok(c) => c,
      Err(e) => panic!("An error occurred while connection to cryptol-remote-api: {}", e),
    };

    match cryptol_client.load_module("SuiteB") {
      Ok(()) => (),
      Err(e) => panic!("Loading module failed: {}", e),
    };

    let arguments = vec!["1 : [16]"];
    match cryptol_client.call("sha384", arguments) {
      Ok(()) => (),
      Err(e) => panic!("SHA384 call failed: {}", e),
    };

    let answer: Answer = serde_json::from_value(cryptol_client.answer).unwrap();
    let sha384_result: SHA384ResultValue = serde_json::from_value(answer.value).unwrap();

    assert_eq!(sha384_result.data, "5d13bb39a64c4ee16e0e8d2e1c13ec4731ff1ac69652c072d0cdc355eb9e0ec41b08aef3dd6fe0541e9fa9e3dcc80f7b");
  }

}
