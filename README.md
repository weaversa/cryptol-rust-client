[![Rust Test](https://github.com/weaversa/cryptol-rust-client/actions/workflows/rust-test.yml/badge.svg)](https://github.com/weaversa/cryptol-rust-client/actions/workflows/rust-test.yml)

# Tests

I am testing the project so far by running the following commands. The first starts cryptol-remote-api. The second runs the tests for the project. I suggest running these commands in separate terminals.

```
$ docker run --rm -it -p 3829:3829 ghcr.io/galoisinc/cryptol-remote-api:nightly +RTS -N -RTS http --host 0.0.0.0 --port 3829 / --max-occupancy 1000
```

```
$ CRYPTOL_SERVER_URL="http://0.0.0.0:3829" cargo test -- --nocapture
$ CRYPTOL_SERVER_URL="http://0.0.0.0:3829" cargo test --example sha384
```
