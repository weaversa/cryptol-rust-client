use std::env;

use serde::{Serialize, Deserialize};
use serde_json::{json, from_str, Value, Error};

use jsonrpsee::core::client::ClientT;
use jsonrpsee::http_client::{ HeaderMap, HeaderValue, HttpClientBuilder };
use jsonrpsee::rpc_params;
use jsonrpsee::core::params::ObjectParams;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

// {"answer":[],"state":"a4909ccf-3ef9-45cc-913b-57e58da75788","stderr":"","stdout":""}}
#[derive(Serialize, Deserialize)]
pub struct CryptolResult {
    answer: Vec<Option<String>>, //serde_json::Value>>,
    state: String,
    stderr: String,
    stdout: String,
}

#[tokio::main]
pub async fn connect() -> Result<u32> {

  let cryptol_server_url = match env::var("CRYPTOL_SERVER_URL") {
    Ok(val) => {
      println!("Attempting to connect to cryptol-remote-api at {}.", val);
      val
    },
    Err(_e) => return Err("connect() failed because CRYPTOL_SERVER_URL environment variable is not set.".into()),
  };

  // Build client
  let mut headers = HeaderMap::new();
  headers.insert("Connection", HeaderValue::from_static("keep-alive"));

  let client = HttpClientBuilder::default()
    .set_headers(headers)
    .build(cryptol_server_url)?;

  let mut params = ObjectParams::new();
  params.insert("module name", "Cryptol");
  params.insert("state", json!(null));

  let response: std::result::Result<CryptolResult, _> =
    client.request("load module"
                  , params
                  ).await;

  println!("state: {}", response.unwrap().state);

  Ok(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_connect_env() {
        match connect() {
          Ok(_val) => (),
          Err(_e) => println!("{}", _e),
        };
        //assert!(connect().is_ok());
    }
}
