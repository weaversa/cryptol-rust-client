use std::env;

use serde::{Serialize, Deserialize};
use serde_json::{json, from_str, Value, Error};

use jsonrpsee::core::client::ClientT;
use jsonrpsee::http_client::{ HeaderMap, HeaderValue, HttpClientBuilder, HttpClient };
use jsonrpsee::rpc_params;
use jsonrpsee::core::params::ObjectParams;

use std::time::Duration;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

/**
 * Cryptol client structure. Contains the active client connection and
 * state attribute.
 */
 
pub struct CryptolClient {
  client: HttpClient,
  state:  String,
}

/**
 * This structure represents the JSON blob returned by cryptol-remote-api.
 * For example:
 *   {"answer":[],"state":"a4909ccf-3ef9-45cc-913b-57e58da75788","stderr":"","stdout":""}
 */

#[derive(Serialize, Deserialize)]
pub struct CryptolResult {
  answer: Vec<Option<String>>, //serde_json::Value>>,
  state:  String,
  stderr: String,
  stdout: String,
}

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
pub async fn connect() -> Result<CryptolClient> {

  // Deduce whether or not `CRYPTOL_SERVER_URL` is defined.
  let cryptol_server_url = match env::var("CRYPTOL_SERVER_URL") {
    Ok(val) => {
      println!("Attempting to connect to cryptol-remote-api at {}.", val);
      val
    },
    Err(_e) => return Err("connect() failed because CRYPTOL_SERVER_URL environment variable is not set.".into()),
  };

  // Insert a 'keep-alive' command into the initial packet
  // header. Perhaps unnecessary?
  let mut headers = HeaderMap::new();
  headers.insert("Connection", HeaderValue::from_static("keep-alive"));

  // Build client, set request timeout to one hour
  let client = HttpClientBuilder::default()
    .set_headers(headers)
    .request_timeout(Duration::from_secs(60 * 60))
    .build(cryptol_server_url)?;

  // Create parameters for loading the Cryptol prelude.
  let mut params = ObjectParams::new();
  params.insert("module name", "Cryptol").unwrap();
  params.insert("state", json!(null)).unwrap();

  // Make a request to cryptol-remote-api to load the Cryptol prelude
  let response: CryptolResult = client.request("load module", params).await?;

  // Create and return a new CryptolClient object to represent the
  // stateful connection
  let cryptol_client = CryptolClient {
                         client: client,
                         state: response.state.clone()
                       };

  Ok(cryptol_client)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_connect() {
      assert!(connect().is_ok());
    }
}
