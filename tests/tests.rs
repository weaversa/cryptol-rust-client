use cryptol_rust_client::CryptolClient;

#[test]
fn test_connect() {
    let cryptol_client = CryptolClient::connect();
    assert!(cryptol_client.is_ok());
}

#[test]
fn test_load_module_success() {
    let mut cryptol_client = match CryptolClient::connect() {
        Ok(c) => c,
        Err(e) => panic!("An error occurred while connecting to cryptol-remote-api: {e}"),
    };

    match cryptol_client.load_module("SuiteB") {
        Ok(_) => (),
        Err(e) => panic!("Loading module failed: {e}"),
    };
}

#[test]
fn test_load_module_failure() {
    let mut cryptol_client = match CryptolClient::connect() {
        Ok(c) => c,
        Err(e) => panic!("An error occurred while connecting to cryptol-remote-api: {e}"),
    };

    assert!(
        cryptol_client.load_module("nosuchmodule").is_err(),
        "nosuchmodule should not exist"
    );
}

#[test]
fn test_call_success() {
    let mut cryptol_client = match CryptolClient::connect() {
        Ok(c) => c,
        Err(e) => panic!("An error occurred while connecting to cryptol-remote-api: {e}"),
    };

    let function = "reverse";
    let arguments = ["[1, 2, 3, 4]"];

    match cryptol_client.call(function, &arguments) {
        Ok(r) => r,
        Err(e) => panic!("An error occured while calling cryptol-remote-api: {e}"),
    };
}

#[test]
fn test_call_failure() {
    let mut cryptol_client = match CryptolClient::connect() {
        Ok(c) => c,
        Err(e) => panic!("An error occurred while connecting to cryptol-remote-api: {e}"),
    };

    let function = "nonsense";
    let arguments = ["[1, 2, 3, 4]"];

    assert!(
        cryptol_client.call(function, &arguments).is_err(),
        "'nonsense' should not be a function in the Cryptol prelude"
    );
}
