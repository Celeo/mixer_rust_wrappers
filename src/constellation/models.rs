use super::errors::ERRORS;
use serde_derive::{Deserialize, Serialize};
use serde_json::Value;
use std::{collections::HashMap, convert::TryFrom, fmt};

#[derive(Debug, Deserialize, Serialize)]
pub struct Event {
    #[serde(rename = "type")]
    pub event_type: String,
    pub event: String,
    pub data: HashMap<String, Value>,
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

#[derive(Debug, Deserialize, Serialize)]
pub struct Method {
    #[serde(rename = "type")]
    pub method_type: String,
    pub method: String,
    pub params: HashMap<String, Value>,
    pub id: usize,
}

impl fmt::Display for Method {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let as_text = serde_json::to_string(&self).expect("Could not convert to JSON");
        write!(f, "{}", as_text)
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct MixerError {
    pub id: u16,
    pub message: String,
}

impl MixerError {
    pub fn explain(&self) -> String {
        match ERRORS.get(&self.id) {
            Some(v) => v.to_owned(),
            None => "Unknown error".to_owned(),
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Reply {
    #[serde(rename = "type")]
    pub reply_type: String,
    pub id: usize,
    pub result: Option<HashMap<String, Value>>,
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

#[derive(Debug)]
pub struct StreamMessage {
    pub event: Option<Event>,
    pub reply: Option<Reply>,
}

impl fmt::Display for StreamMessage {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let e = match &self.event {
            Some(e) => format!("{}", e),
            None => String::from("None"),
        };
        let r = match &self.reply {
            Some(r) => format!("{}", r),
            None => String::from("None"),
        };
        write!(f, "{} | {}", e, r)
    }
}
