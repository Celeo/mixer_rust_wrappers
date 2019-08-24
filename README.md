# Mixer Rust Wrappers

[![CircleCI](https://circleci.com/gh/Celeo/mixer_rust_wrappers.svg?style=svg)](https://circleci.com/gh/Celeo/mixer_rust_wrappers)

Rust wrappers for the Mixer APIs at https://dev.mixer.com/.

## Building

### Requirements

* Git
* A recent version of [Rust](https://www.rust-lang.org/tools/install)

### Steps

```sh
git clone https://github.com/Celeo/mixer_rust_wrappers
cd mixer_rust_wrappers
cargo build
```

Run tests with `cargo test`.

## Using

Add the [most recent version](https://crates.io/crates/mixer_wrappers) to your Cargo.toml and build.

This library is split into 2 parts: a tiny convenience wrapper for the REST API, and a wrapper for the Constellation real-time API.

### REST

The REST wrapper is very simple. It basically only does two things:

1. Handle sending the 'client-id' header
1. Return an error for non-successful HTTP requests (outside of 2XX)

Create an instance of the `REST` struct with

```rust
use mixer_wrappers::REST;

let client = REST::new("your_client_id_here");
```

If you don't know your Client ID, you can get it from [Mixer](https://mixer.com/lab/keypopup).

Send API requests with

```rust
use reqwest::Method;

let resp_text = client.query(Method::GET, "some/endpoint", None, None).unwrap();
```

The `query` function returns a Result, so be sure to handle that appropriately.

### Constellation

This is the more interesting part of the library. This wrapper makes it easy to listen to and send messages to the real-time API. Start with

```rust
use mixer_wrappers::connect;

let (mut client, receiver) = connect().unwrap();
```

Note that at this call, you'll need to have a 'CLIENT_ID' environment variable set, and that the `connect` method returns a Result, as expected.

You can see a full example of what do do with these variables in the docs.

## License

Licensed under either of

* Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE))
* MIT license ([LICENSE-MIT](LICENSE-MIT))

## Contributing

Please feel free to contribute. Please open an issue first (or comment on an existing one) so that I know that you want to add/change something.

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
