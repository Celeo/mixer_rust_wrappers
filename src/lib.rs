//! Rust wrappers for the Mixer APIs.
//!
//! Use the `constellation` module for communicating asynchronously with the real-time
//! [Constellation] endpoint, and the `rest` module for communicating synchronously with
//! the [Core REST API].
//!
//! [Constellation]: https://dev.mixer.com/reference/constellation
//! [Core REST API]: https://dev.mixer.com/rest/index.html

#![warn(missing_docs)]

/// Chat API
pub mod chat;
/// Real-time API
pub mod constellation;
/// Internal wrappers around ws-rs for convenience
mod internal;
/// OAuth helpers
pub mod oauth;
/// REST API
pub mod rest;

pub use chat::ChatClient;
pub use constellation::ConstellationClient;
pub use rest::REST;
