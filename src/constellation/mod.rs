/// Static models for the JSON data
pub mod models;

use crate::internal::{connect as socket_connect, ClientSocketWrapper};
use atomic_counter::AtomicCounter;
use failure::{format_err, Error};
use log::debug;
use serde_json::Value;
use std::{collections::HashMap, convert::TryFrom, sync::mpsc::Receiver};

use models::{Event, Method, Reply};

/// Possible messages from the socket.
pub enum StreamMessage {
    /// Event types
    Event(Event),
    /// Reply types
    Reply(Reply),
}

/// Wrapper for connecting and interacting with Constellation.
pub struct ConstellationClient {
    client: ClientSocketWrapper,
}

impl ConstellationClient {
    /// Connect to Constellation.
    ///
    /// # Arguments
    ///
    /// * `client_id` - your client ID
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use mixer_wrappers::ConstellationClient;
    /// let (client, receiver) = ConstellationClient::connect("aaa").unwrap();
    /// ```
    pub fn connect(client_id: &str) -> Result<(Self, Receiver<String>), Error> {
        let (client, receiver) = socket_connect("wss://constellation.mixer.com", client_id)?;
        Ok((ConstellationClient { client }, receiver))
    }

    /// Call a method, sending data to the socket.
    ///
    /// # Arguments
    ///
    /// * `method` - method name
    /// * `params` - method parameters
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use mixer_wrappers::ConstellationClient;
    /// # use serde_json::{json, Value};
    /// # use std::collections::HashMap;
    /// # let (mut client, _) = ConstellationClient::connect("").unwrap();
    /// let mut map = HashMap::new();
    /// map.insert(String::from("abc"), json!(123));
    /// if let Err(e) = client.call_method("some_method", &map) {
    ///     // ...
    /// }
    /// ```
    pub fn call_method(
        &mut self,
        method: &str,
        params: &HashMap<String, Value>,
    ) -> Result<(), Error> {
        if !self.client.check_connection() {
            return Err(format_err!("Not connected to socket"));
        }
        let to_send = Method {
            method_type: "method".to_owned(),
            method: method.to_owned(),
            params: params.to_owned(),
            id: self.client.method_counter.inc(),
        };
        debug!("Sending method call to socket: {:?}", to_send);
        self.client
            .socket_out
            .send(serde_json::to_string(&to_send)?)?;
        Ok(())
    }

    /// Helper method to parse the JSON messages into structs.
    ///
    /// # Arguments
    ///
    /// * `message` - String message from the receiver
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use mixer_wrappers::ConstellationClient;
    /// # let (client, receiver) = ConstellationClient::connect("").unwrap();
    /// let message = client.parse(&receiver.recv().unwrap()).unwrap();
    /// ```
    pub fn parse(&self, message: &str) -> Result<StreamMessage, Error> {
        let json: Value = serde_json::from_str(message)?;
        let type_ = match json["type"].as_str() {
            Some(t) => t,
            None => return Err(format_err!("Message does not have a 'type' field")),
        };
        if type_ == "event" {
            return match Event::try_from(json.clone()) {
                Ok(e) => Ok(StreamMessage::Event(e)),
                Err(e) => Err(format_err!("{}", e)),
            };
        }
        if type_ == "reply" {
            return match Reply::try_from(json.clone()) {
                Ok(r) => Ok(StreamMessage::Reply(r)),
                Err(e) => Err(format_err!("{}", e)),
            };
        }
        Err(format_err!("Unknown type '{}'", type_))
    }
}
