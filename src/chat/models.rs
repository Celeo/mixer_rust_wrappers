use serde_derive::{Deserialize, Serialize};
use serde_json::Value;
use std::{collections::HashMap, convert::TryFrom};

/// An Event coming in from the socket.
///
/// These are sent from the chat server when connecting,
/// receiving a live event, etc.
///
/// See https://dev.mixer.com/reference/chat/events
#[derive(Debug, Deserialize, Serialize)]
pub struct Event {
    /// Always 'event'
    #[serde(rename = "type")]
    pub event_type: String,
    /// Which event
    pub event: String,
    /// Data associated with the event. Not that this is,
    /// per the docs, completely unstructured; it depends
    /// on which kind of event was received.
    pub data: Option<Value>,
}

impl TryFrom<Value> for Event {
    type Error = &'static str;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        let as_text = serde_json::to_string(&value).unwrap();
        let event: Event = match serde_json::from_str(&as_text) {
            Ok(r) => r,
            Err(_) => return Err("Could not load from JSON"),
        };
        Ok(event)
    }
}

/// A Method to send to the socket.
///
/// This is how clients send data _to_ the socket.
///
/// See https://dev.mixer.com/reference/chat/methods#methods
#[derive(Debug, Deserialize, Serialize)]
pub struct Method {
    /// Always 'method'
    #[serde(rename = "type")]
    pub method_type: String,
    /// The method to call
    pub method: String,
    /// Method's parameters
    pub arguments: HashMap<String, Value>,
    /// Unique id for this method call
    pub id: usize,
}

/// A Replay to a method call.
///
/// These are sent from the chat server to the client as
/// a response to the client sending a method.
///
/// See https://dev.mixer.com/reference/chat/methods#reply
#[derive(Debug, Deserialize, Serialize)]
pub struct Reply {
    #[serde(rename = "type")]
    /// Which method type this reply is for
    pub reply_type: String,
    /// The id of the method this reply is for
    pub id: usize,
    /// Method call result
    pub data: Option<HashMap<String, Value>>,
    /// Method error
    pub error: Option<String>,
}

impl TryFrom<Value> for Reply {
    type Error = &'static str;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        let as_text = serde_json::to_string(&value).unwrap();
        let reply: Reply = match serde_json::from_str(&as_text) {
            Ok(r) => r,
            Err(_) => return Err("Could not load from JSON"),
        };
        Ok(reply)
    }
}

/// Wrapper for either an Event, or a Reply.
///
/// This is the struct that's sent through the returned
/// MPSC Receiver when connecting.
#[derive(Debug)]
pub struct StreamMessage {
    /// Potential event
    pub event: Option<Event>,
    /// Potential reply
    pub reply: Option<Reply>,
}

#[cfg(test)]
mod tests {}
