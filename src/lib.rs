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
  answer: Vec<Option<String>>, //serde_json::Value>>,
  state:  String,
  stderr: String,
  stdout: String,
}

/**
 * Cryptol client struct. Contains the active client connection and
 * state attribute.
 */

#[derive(Debug)]
struct CryptolClient {
  client: HttpClient,
  state:  String,
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
  async fn connect() -> Result<Self> {

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
      .request_timeout(Duration::from_secs(60 * 60)) // Set longer request timeout
      .build(cryptol_server_url)?;

    // Create parameters for loading the Cryptol prelude.
    let mut params = ObjectParams::new();
    params.insert("module name", "Cryptol").unwrap();
    params.insert("state", json!(null)).unwrap();

    // Make a request to cryptol-remote-api to load the Cryptol prelude
    let response: CryptolResult = client.request("load module", params).await?;

    // Create and return a new CryptolClient object to represent the
    // stateful connection
    Ok(CryptolClient { client: client, state: response.state.clone() })
  }

}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_connect() {
      let mut cryptol_client = CryptolClient::connect();
      assert!(cryptol_client.is_ok());
      println!("{:?}", cryptol_client);
    }
}
