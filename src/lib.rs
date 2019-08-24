//! Rust wrappers for the Mixer APIs.
//!
//! Use the `constellation` module for communicating asynchronously with the real-time
//! [Constellation] endpoint, and the `rest` module for communicating synchronously with
//! the [Core REST API].
//!
//! [Constellation]: https://dev.mixer.com/reference/constellation
//! [Core REST API]: https://dev.mixer.com/rest/index.html

#![warn(missing_docs)]

/// Real-time API
pub mod constellation;
/// REST API
pub mod rest;

pub use constellation::{connect, ConstellationClient};
pub use rest::REST;
