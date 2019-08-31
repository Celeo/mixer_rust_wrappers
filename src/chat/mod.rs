/// Static models for JSON data
pub mod models;

use crate::internal::{connect as socket_connect, ClientSocketWrapper};
use atomic_counter::AtomicCounter;
use failure::{format_err, Error};
use log::debug;
use serde_json::{json, Value};
use std::{convert::TryFrom, sync::mpsc::Receiver, thread::JoinHandle};

use models::{Event, Method, Reply};

/// Possible messages from the socket.
pub enum StreamMessage {
    /// Event types
    Event(Event),
    /// Reply types
    Reply(Reply),
}

/// Wrapper for connecting and interacting with the chat server.
pub struct ChatClient {
    client: ClientSocketWrapper,
    /// Internal thread join handle
    pub join_handle: JoinHandle<()>,
}

impl ChatClient {
    /// Connect to the chat server.
    ///
    /// Per the [documentation], connecting to the chat server isn't as
    /// straightforward as connecting to Constellation, as the client
    /// must first make a call to the REST API to fetch information about
    /// the chat connection, including the endpoint to connect to. This
    /// function does not handle that process; use the REST API included
    /// in this crate to get that information.
    ///
    /// # Arguments
    ///
    /// * `endpoint` - chat websocket endpoint to connect to
    /// * `client_id` - your client ID
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use mixer_wrappers::ChatClient;
    /// let (mut client, receiver) = ChatClient::connect("aaa", "bbb").unwrap();
    /// ```
    ///
    /// [documentation]: https://dev.mixer.com/reference/chat/connection
    pub fn connect(endpoint: &str, client_id: &str) -> Result<(Self, Receiver<String>), Error> {
        let (client, join_handle, receiver) = socket_connect(endpoint, client_id)?;
        Ok((
            ChatClient {
                client,
                join_handle,
            },
            receiver,
        ))
    }

    /// Authenticate with the server. This must be done after connecting.
    ///
    /// Per the [documentation], you can either authenticate anonymously,
    /// or as an actual user. The former is done by passing this function
    /// `None`s for the second and third parameters.
    ///
    /// # Arguments
    /// * `channel_id` - channel to connect to, fetched from the [REST API]
    /// * `user_id` - Option of user to auth as
    /// * `auth_key` - Option of user key to use
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use mixer_wrappers::ChatClient;
    /// # let (mut client, _) = ChatClient::connect("aaa", "bbb").unwrap();
    /// if let Err(e) = client.authenticate(123, Some(456), Some("ccc")) {
    ///     // ...
    /// }
    /// ```
    ///
    /// [documentation]: https://dev.mixer.com/reference/chat/methods/auth
    /// [REST API]: https://dev.mixer.com/reference/chat/connection
    pub fn authenticate(
        &mut self,
        channel_id: usize,
        user_id: Option<usize>,
        auth_key: Option<&str>,
    ) -> Result<(), Error> {
        let method = if user_id.is_none() || auth_key.is_none() {
            debug!("Authenticating as anonymous");
            Method {
                method_type: "method".to_owned(),
                method: "auth".to_owned(),
                arguments: vec![json!(channel_id)],
                id: self.client.method_counter.inc(),
            }
        } else {
            debug!("Authenticating as a user");
            Method {
                method_type: "method".to_owned(),
                method: "auth".to_owned(),
                arguments: vec![
                    json!(channel_id),
                    json!(user_id.unwrap()),
                    json!(auth_key.unwrap()),
                ],
                id: self.client.method_counter.inc(),
            }
        };
        self.client
            .socket_out
            .send(serde_json::to_string(&method)?)?;
        Ok(())
    }

    /// Call a method, sending data to the socket.
    ///
    /// The `arguments` parameter is so dynamic because while the arguments
    /// parameter is an array, it's JSON, so there can be any number of elements
    /// in the array of different types.
    ///
    /// # Arguments
    ///
    /// * `method` - method name
    /// * `arguments` - method arguments
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use mixer_wrappers::ChatClient;
    /// # use serde_json::{json, Value};
    /// # let (mut client, _) = ChatClient::connect("", "").unwrap();
    /// if let Err(e) = client.call_method("some_method", &[json!(123), json!("abc")]) {
    ///     // ...
    /// }
    /// ```
    pub fn call_method(&mut self, method: &str, arguments: &[Value]) -> Result<(), Error> {
        if !self.client.check_connection() {
            return Err(format_err!("Not connected to socket"));
        }
        let to_send = Method {
            method_type: "method".to_owned(),
            method: method.to_owned(),
            arguments: arguments.to_owned(),
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
    /// # use mixer_wrappers::ChatClient;
    /// let message = ChatClient::parse("{\"type\":\"event\"...}").unwrap();
    /// ```
    pub fn parse(message: &str) -> Result<StreamMessage, Error> {
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
