# Rust Cryptol Client

[![Build, Test, Publish](https://github.com/weaversa/cryptol-rust-client/actions/workflows/main.yml/badge.svg)](https://github.com/weaversa/cryptol-rust-client/actions/workflows/main.yml)
[![Docker](https://github.com/weaversa/cryptol-rust-client/actions/workflows/docker.yml/badge.svg)](https://github.com/weaversa/cryptol-rust-client/actions/workflows/docker.yml)

# Purpose

This crate is a collection of utilities for connecting to and
interacting with a running
[`cryptol-remote-api`](https://github.com/GaloisInc/cryptol/tree/master/cryptol-remote-api)
instance.

# Assurance

This project uses a number of mechanisms for increasing its assurance.

  - #![forbid(unsafe_code)] is used to ensure the use of safe Rust,
  - the [`clippy`](https://github.com/rust-lang/rust-clippy) linter is
    used at the pedantic level,
  - the [rust formatter](https://github.com/rust-lang/rustfmt) is used
    to ensure the code adheres to idomatic Rust,
  - every public function has a postive and negative test,
  - the above tools are used by the CI to enforce invariants on this project.

# Local Testing

Presuming Docker is available, the project may be tested by first
starting `cryptol-remote-api`.

```
$ docker run --rm -it -p 49352:49352 ghcr.io/galoisinc/cryptol-remote-api:nightly +RTS -N -RTS http --host 0.0.0.0 --port 49352 / --max-occupancy 1000
```

Next, the following commands may be run to test this project:

```
$ CRYPTOL_SERVER_URL="http://0.0.0.0:49352" cargo test
$ CRYPTOL_SERVER_URL="http://0.0.0.0:49352" cargo test --example sha384
```

As well, one can run the provided SHA-384 example as follows:

```
$ CRYPTOL_SERVER_URL="http://0.0.0.0:49352" cargo run --example sha384 "0x12345678"
```
