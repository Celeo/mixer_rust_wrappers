use super::errors::ERRORS;
use serde_derive::{Deserialize, Serialize};
use serde_json::Value;
use std::{collections::HashMap, convert::TryFrom, fmt};

/// An Event coming in from the socket.
///
/// These are sent from Constellation when connecting,
/// receiving a live event, etc.
///
/// See https://dev.mixer.com/reference/constellation/events
#[derive(Debug, Deserialize, Serialize)]
pub struct Event {
    #[serde(rename = "type")]
    /// Always 'type'
    pub event_type: String,
    /// Which event
    pub event: String,
    /// Data associated with the event. Not that this is,
    /// per the docs, completely unstructured; it depends
    /// on which kind of event was received.
    pub data: Option<Value>,
}

impl fmt::Display for Event {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let as_text = serde_json::to_string(&self).expect("Could not convert to JSON");
        write!(f, "{}", as_text)
    }
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
/// See https://dev.mixer.com/reference/constellation/methods
#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub struct Method {
    #[serde(rename = "type")]
    /// Always 'method'
    pub method_type: String,
    /// The method to call
    pub method: String,
    /// Method's parameters
    pub params: HashMap<String, Value>,
    /// Unique id for this method call
    pub id: usize,
}

impl fmt::Display for Method {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let as_text = serde_json::to_string(&self).expect("Could not convert to JSON");
        write!(f, "{}", as_text)
    }
}

/// Error from Constellation
#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub struct MixerError {
    /// Error's id
    pub id: u16,
    /// Error's message
    pub message: String,
}

impl MixerError {
    /// Look up the error's explanation
    pub fn explain(&self) -> String {
        match ERRORS.get(&self.id) {
            Some(v) => v.to_owned(),
            None => "Unknown error".to_owned(),
        }
    }
}

/// A Replay to a method call.
///
/// These are sent from Constellation to the client as
/// a response to the client sending a method.
#[derive(Debug, Deserialize, Serialize)]
pub struct Reply {
    #[serde(rename = "type")]
    /// Which method type this reply is for
    pub reply_type: String,
    /// The id of the method this reply is for
    pub id: usize,
    /// Method call result
    pub result: Option<HashMap<String, Value>>,
    /// Method error
    pub error: Option<MixerError>,
}

impl fmt::Display for Reply {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let as_text = serde_json::to_string(&self).expect("Could not convert to JSON");
        write!(f, "{}", as_text)
    }
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
mod tests {
    use super::{Event, Method, MixerError, Reply};
    use serde_json::{json, Value};
    use std::{collections::HashMap, convert::TryFrom};

    #[test]
    fn event_try_from_json() {
        let text = r#"{"type":"event","event":"foobar","data": null}"#;
        let json: Value = serde_json::from_str(&text).unwrap();
        let event = Event::try_from(json).unwrap();

        assert_eq!(event.event, "foobar");
    }

    #[test]
    fn event_try_from_json_fail() {
        let json = json!({});
        let res = Event::try_from(json);

        assert!(res.is_err());
    }

    #[test]
    fn reply_try_from_json() {
        let text = r#"{"type":"reply","id":40,"result":null,"error":null}"#;
        let json: Value = serde_json::from_str(&text).unwrap();
        let reply = Reply::try_from(json).unwrap();

        assert_eq!(reply.id, 40);
    }

    #[test]
    fn reply_try_from_json_fail() {
        let json = json!({});
        let res = Reply::try_from(json);

        assert!(res.is_err());
    }

    #[test]
    fn error_explain() {
        let error = MixerError {
            id: 4008,
            message: String::new(),
        };

        assert_eq!(error.explain(), "Unknown packet type".to_owned());
    }

    #[test]
    fn error_explain_bad_id() {
        let error = MixerError {
            id: 0,
            message: String::new(),
        };

        assert_eq!(error.explain(), "Unknown error".to_owned());
    }

    #[test]
    fn event_from_json() {
        let text = r#"{"type":"event","event":"hello","data":{}}"#;
        let event: Event = serde_json::from_str(&text).unwrap();

        assert_eq!("event", event.event_type);
        assert_eq!("hello", event.event);
        assert_eq!(Some(json!({})), event.data);

        assert_eq!(text, format!("{}", event));
    }

    #[test]
    fn reply_from_json() {
        let text = r#"{"type":"reply","id":100,"result":{"foo":123},"error":null}"#;
        let reply: Reply = serde_json::from_str(&text).unwrap();

        assert_eq!("reply", reply.reply_type);
        assert_eq!(100, reply.id);
        let mut map = HashMap::new();
        map.insert(String::from("foo"), json!(123));
        assert_eq!(Some(map), reply.result);
        assert_eq!(None, reply.error);

        assert_eq!(text, format!("{}", reply));
    }

    #[test]
    fn method_display() {
        let mut map = HashMap::new();
        map.insert(String::from("foo"), json!("bar"));
        let method = Method {
            method_type: String::from("method"),
            method: String::from("something"),
            params: map,
            id: 100,
        };
        let as_text = format!("{}", method);
        let method_check = serde_json::from_str(&as_text).unwrap();
        assert_eq!(method, method_check);
    }
}
