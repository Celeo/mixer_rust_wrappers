use serde_derive::{Deserialize, Serialize};
use serde_json::Value;
use std::{collections::HashMap, convert::TryFrom};

/// An Event coming in from the socket.
///
/// These are sent from Constellation when connecting,
/// receiving a live event, etc.
///
/// See https://dev.mixer.com/reference/constellation/events
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
/// See https://dev.mixer.com/reference/constellation/methods
#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub struct Method {
    /// Always 'method'
    #[serde(rename = "type")]
    pub method_type: String,
    /// The method to call
    pub method: String,
    /// Method's parameters
    pub params: HashMap<String, Value>,
    /// Unique id for this method call
    pub id: usize,
}

/// Error from Constellation
#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub struct MixerError {
    /// Error's id
    pub id: u16,
    /// Error's message
    pub message: String,
}

/// A Replay to a method call.
///
/// These are sent from Constellation to the client as
/// a response to the client sending a method.
///
/// See https://dev.mixer.com/reference/constellation/methods#reply
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

#[cfg(test)]
mod tests {
    use super::{Event, MixerError, Reply};
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
    fn event_from_json() {
        let text = r#"{"type":"event","event":"hello","data":{}}"#;
        let event: Event = serde_json::from_str(&text).unwrap();

        assert_eq!("event", event.event_type);
        assert_eq!("hello", event.event);
        assert_eq!(Some(json!({})), event.data);

        assert_eq!(text, serde_json::to_string(&event).unwrap());
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

        assert_eq!(text, serde_json::to_string(&reply).unwrap());
    }

    #[test]
    fn test_mixer_error() {
        let err = MixerError {
            id: 100,
            message: "".to_owned(),
        };
        let _ = format!("{:?}", err);
    }
}
