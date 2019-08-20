use lazy_static::lazy_static;
use std::collections::HashMap;

lazy_static! {
    pub static ref ERRORS: HashMap<u16, String> = {
        let mut m = HashMap::new();
        m.insert(
            1011,
            "Sent in a close or method reply if an unknown internal error occurs.".to_owned(),
        );
        m.insert(1012, "Sent in a close frame when we deploy or restart Constellation; clients should attempt to reconnect.".to_owned());
        m.insert(4006, "Error parsing payload as JSON".to_owned());
        m.insert(
            4007,
            "Error decompressing a supposedly-gzipped payload".to_owned(),
        );
        m.insert(4008, "Unknown packet type".to_owned());
        m.insert(4009, "Unknown method name call".to_owned());
        m.insert(
            4010,
            "Error parsing method arguments (not the right type or structure)".to_owned(),
        );
        m.insert(4011, "The user session has expired; if using a cookie, they should log in again, or get a bearer auth token if using an authorization header.".to_owned());
        m.insert(
            4106,
            "Unknown event used in a livesubscribe call".to_owned(),
        );
        m.insert(
            4107,
            "You do not have access to subscribe to that livesubscribe event".to_owned(),
        );
        m.insert(
            4108,
            "You are already subscribed to that livesubscribe event (during livesubscribe)"
                .to_owned(),
        );
        m.insert(4109, "You are not subscribed to that livesubscribe event (in response to a liveunsubscribe method)".to_owned());
        m.insert(4110, "You cannot make more subscriptions (in response to a livesubscribe method). See liveloading limits.".to_owned());
        m
    };
}
