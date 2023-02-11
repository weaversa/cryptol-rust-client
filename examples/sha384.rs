//! # SHA384
//!
//! This is a demonstration of how to use the `cryptol-rust-client`
//! crate to call a Cryptol function via the `cryptol-remote-api`.

use cryptol_rust_client::CryptolClient;
use serde::{Deserialize, Serialize};
use std::env;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

/// This is the JSON blob representing a response for a call to SHA384.
///
/// For example:
///   `{"data":"5d13bb39a64c4ee16e0e8d2e1c13ec4731ff1ac69652c072d0cdc355eb9e0ec41b08aef3dd6fe0541e9fa9e3dcc80f7b","encoding":"hex","expression":"bits","width":384}`

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SHA384ResultValue {
    data: String,
    encoding: String,
    expression: String,
    width: i64,
}

/// This function calls SHA384 via `cryptol-remote-api`.

fn sha384(mut cryptol_client: CryptolClient, input: &str) -> Result<String> {
    // Load Cryptol's `SuiteB` module.
    cryptol_client.load_module("SuiteB")?;

    // Add the input to the list of input parameters and call Cryptol's `sha384`.
    let arguments = vec![input];
    let answer = cryptol_client.call("sha384", arguments)?;

    // Transform the resulting JSON into a `SHA384ResultValue` type.
    let sha384_result: SHA384ResultValue = serde_json::from_value(answer.value).unwrap();

    // Prepend '0x' to the resulting hex string.
    Ok(format!("0x{}", sha384_result.data))
}

/// This is the `main` function for this example. Here are some sample
/// inputs:
///
/// `cargo run --example sha384 "(join \"Hello World\")"`
///
/// `cargo run --example sha384 "0x1234"`

fn main() {
    let args: Vec<String> = env::args().collect();

    let value_to_hash = &args[1];

    let cryptol_client = match CryptolClient::connect() {
        Ok(c) => c,
        Err(e) => panic!("An error occurred while connecting to cryptol-remote-api: {e}"),
    };

    println!("Calling SHA-384 on {value_to_hash}");

    let result = match sha384(cryptol_client, value_to_hash) {
        Ok(r) => r,
        Err(e) => panic!("An error occured while calling sha384: {e}"),
    };

    println!("Hash: {result}");
}

#[cfg(test)]
mod sha384_tests {
    use super::*;

    #[test]
    fn test_call_sha384_success() {
        let cryptol_client = match CryptolClient::connect() {
            Ok(c) => c,
            Err(e) => panic!("An error occurred while connecting to cryptol-remote-api: {e}"),
        };

        let result = match sha384(cryptol_client, "0x0001") {
            Ok(r) => r,
            Err(e) => panic!("An error occured while calling sha384: {e}"),
        };

        assert_eq!(result, "0x5d13bb39a64c4ee16e0e8d2e1c13ec4731ff1ac69652c072d0cdc355eb9e0ec41b08aef3dd6fe0541e9fa9e3dcc80f7b");
    }

    #[test]
    fn test_call_sha384_failure() {
        let cryptol_client = match CryptolClient::connect() {
            Ok(c) => c,
            Err(e) => panic!("An error occurred while connecting to cryptol-remote-api: {e}"),
        };

        match sha384(cryptol_client, "not a number") {
            Ok(_) => panic!("'not a number' should not type correctly as an argument to sha384"),
            Err(_) => (),
        }
    }
}
