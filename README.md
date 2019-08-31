# Mixer Rust Wrappers

[![CircleCI](https://circleci.com/gh/Celeo/mixer_rust_wrappers.svg?style=svg)](https://circleci.com/gh/Celeo/mixer_rust_wrappers)
[![Crates.io](https://img.shields.io/crates/v/mixer_wrappers)](https://crates.io/crates/mixer_wrappers)
[![Docs.rs](https://docs.rs/mixer_wrappers/badge.svg)](https://docs.rs/mixer_wrappers/latest/mixer_wrappers/)

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

### Tests

Run tests with `cargo test`.

If you want code coverage, you can use [kcov](https://github.com/SimonKagstrom/kcov) via the included `./get_coverage.sh` script.

This doesn't work well on CI, so there's no badge.

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

This wrapper makes it easy to listen to and send messages to the real-time API. Start with

```rust
use mixer_wrappers::ConstellationClient;

let (mut client, receiver) = ConstellationClient::connect("your_client_id_here").unwrap();
```

Note that the `connect` method returns a Result.

You can see a more full example of what do do with these variables in the docs.

### Chat

This wrapper makes it easy to listen to and send messages to the chat server. Since the chat server
doesn't have a specific, constant endpoint, connecting is a bit more involved.

Start with

```rust
use mixer_wrappers::{ChatClient, REST};

let api = REST::new(&client_id);
let chat_helper = api.chat_helper();
let channel_id = chat_helper.get_channel_id("your_username_here").unwrap();
let endpoints = chat_helper.get_servers(channel_id).unwrap();
let (mut client, receiver) = Chat::connect(endpoints[0], "your_client_id_here").unwrap();
```

Note that the `connect` method returns a Result.

You can see a more full example of what do do with these variables in the docs.

## Examples

There are examples of each methods use in the documentation. For something a little more complete, look at the doc comments on the methods in the code.

For complete examples, see the `./examples` directory.

## License

Licensed under either of

* Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE))
* MIT license ([LICENSE-MIT](LICENSE-MIT))

## Contributing

Please feel free to contribute. Please open an issue first (or comment on an existing one) so that I know that you want to add/change something.

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license,
shall be dual licensed as above, without any additional terms or conditions.
