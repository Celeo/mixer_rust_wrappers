#![allow(unused)] // FIXME

use atomic_counter::{AtomicCounter, ConsistentCounter};
use failure::{format_err, Error};
use log::{debug, error, info, warn};
use serde::Deserialize;
use serde_json::Value;
use std::{
    collections::HashMap,
    sync::mpsc::{channel, Receiver, Sender as ChanSender},
    thread::{self, JoinHandle},
};
use url::Url;
use ws::{
    connect as socket_connect, CloseCode, Error as SocketError, Handler, Handshake,
    Message as SocketMessage, Request, Result as WSResult, Sender as SocketSender,
};

struct RawSocketWrapper {
    client_id: String,
    connection_sender: ChanSender<bool>,
    message_sender: ChanSender<String>,
}

impl RawSocketWrapper {
    /// Create a new low-level client.
    fn new(
        client_id: &str,
        connection_sender: ChanSender<bool>,
        message_sender: ChanSender<String>,
    ) -> Self {
        RawSocketWrapper {
            client_id: client_id.to_owned(),
            connection_sender,
            message_sender,
        }
    }
}

impl Handler for RawSocketWrapper {
    /// Overrides the default request builder to pass in the client-id header.
    fn build_request(&mut self, url: &Url) -> WSResult<Request> {
        let mut req = Request::from_url(url)?;
        // the two required headers: client-id and x-is-bot
        req.headers_mut()
            .push(("client-id".into(), self.client_id.clone().into()));
        req.headers_mut().push(("x-is-bot".into(), "true".into()));
        Ok(req)
    }

    /// Handler for when the connection is opened.
    fn on_open(&mut self, _handshake: Handshake) -> WSResult<()> {
        info!("Connected");
        self.connection_sender.send(true).unwrap();
        Ok(())
    }

    /// Handler for when the connection receives a message.
    fn on_message(&mut self, msg: SocketMessage) -> WSResult<()> {
        if !msg.is_empty() && msg.is_text() {
            debug!("Got message from socket: {:?}", msg);
            self.message_sender
                .send(msg.as_text().unwrap().to_owned())
                .unwrap();
        }
        Ok(())
    }

    /// Handler for when the connection is closed.
    fn on_close(&mut self, code: CloseCode, reason: &str) {
        warn!("Closed: {:?} | {}", code, reason);
        self.connection_sender.send(false).unwrap();
    }

    /// Handler for when the connection receives an error.
    fn on_error(&mut self, error: SocketError) {
        error!("An error occurred: {}", error);
    }
}

/// Client for communicating with Mixer's Constellation endpoint.
pub struct ClientSocketWrapper {
    socket_out: SocketSender,
    connection_receiver: Receiver<bool>,
    /// Thread handle that you can join to to keep your program running
    pub client_thread_handler: JoinHandle<()>,
    is_connected: bool,
    method_counter: ConsistentCounter,
}

impl ClientSocketWrapper {
    /// Create a new high-level client.
    fn new(
        socket_out: SocketSender,
        connection_receiver: Receiver<bool>,
        client_thread_handler: JoinHandle<()>,
    ) -> Self {
        ClientSocketWrapper {
            socket_out,
            connection_receiver,
            client_thread_handler,
            is_connected: false,
            method_counter: ConsistentCounter::new(0),
        }
    }

    /// Checks to see if new connection status has come from the underlying client.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let is_connected = client.check_connection();
    /// ```
    pub fn check_connection(&mut self) -> bool {
        match self.connection_receiver.try_recv() {
            Ok(v) => {
                debug!("Got new connection status: {}", v);
                self.is_connected = v;
                self.is_connected
            }
            Err(_) => self.is_connected,
        }
    }

    /// Send a raw message through the socket connection.
    ///
    /// # Arguments
    ///
    /// * `message` - raw message to send
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// client.send_message("Hello world").unwrap();
    /// ```
    pub fn send_raw_message(&mut self, message: &str) -> Result<(), Error> {
        if !self.check_connection() {
            return Err(format_err!("Not connected to socket"));
        }
        self.socket_out.send(message)?;
        Ok(())
    }
}

/// Create a connection to the Mixer socket endpoint.
///
/// Returns a tuple of the client you can use to send data to the server,
/// and an MPSC Receiver used for getting data out of the socket. This method
/// utilizes threads so that it does not block; the program can continue
/// running after calling this method.
///
/// Of the tuple that's returned, the first struct is the client that is
/// used to send messages to the server. The second item is the MPSC
/// receiver that is sent the replies and events back from the socket.
/// Handling these structs is a task for the program.
///
/// # Arguments
///
/// * `endpoint` - server socket endpoint
/// * `client_id` - client ID
///
/// # Examples
///
/// ## Simple method call
///
/// ```rust,no_run
/// # use mixer_wrappers::internal::connect;
/// let (client, receiver) = connect("wss://somewhere.com:443", "aaaaaaaaaa").unwrap();
/// ```
pub fn connect(
    endpoint: &str,
    client_id: &str,
) -> Result<(ClientSocketWrapper, Receiver<String>), Error> {
    debug!("Setting up connection");
    // create channels
    let (ws_send, ws_recv) = channel::<SocketSender>();
    let (conn_send, conn_recv) = channel::<bool>();
    let (msg_send, msg_rev) = channel::<String>();

    // launch the socket connection in a new thread
    let endpoint = endpoint.to_owned();
    let client_id = client_id.to_owned();
    let client_handler = thread::spawn(move || {
        debug!("Starting connection");
        socket_connect(endpoint, |socket_out| {
            let client = RawSocketWrapper::new(&client_id, conn_send.clone(), msg_send.clone());
            // send the socket output struct through the corresponding channel
            ws_send
                .send(socket_out)
                .expect("Could not send socket output to channel");
            client
        })
        .expect("Could not start socket connection");
    });
    // receive the socket output struct
    let socket_out = ws_recv.recv()?;

    // create the final client
    let client = ClientSocketWrapper::new(socket_out, conn_recv, client_handler);

    // return the final client
    debug!("Connection setup finished");
    Ok((client, msg_rev))
}

pub mod constellation {

    use super::{connect, ClientSocketWrapper};
    use crate::constellation::models::{Event, Method, Reply};
    use atomic_counter::{AtomicCounter, ConsistentCounter};
    use failure::{format_err, Error};
    use log::debug;
    use serde_json::Value;
    use std::{collections::HashMap, convert::TryFrom, sync::mpsc::Receiver};

    pub enum StreamMessage {
        Event(Event),
        Reply(Reply),
    }

    struct ConstellationClient {
        client: ClientSocketWrapper,
        method_counter: ConsistentCounter,
    }

    impl ConstellationClient {
        fn connect(client_id: &str) -> Result<(Self, Receiver<String>), Error> {
            let (client, receiver) = super::connect("wss://constellation.mixer.com", client_id)?;
            Ok((
                ConstellationClient {
                    client,
                    method_counter: ConsistentCounter::new(0),
                },
                receiver,
            ))
        }

        pub fn create_method(&mut self, method: &str, params: &HashMap<String, Value>) -> Method {
            Method {
                method_type: "method".to_owned(),
                method: method.to_owned(),
                params: params.clone(),
                id: self.method_counter.inc(),
            }
        }

        pub fn call_method(
            &mut self,
            method: &str,
            params: &HashMap<String, Value>,
        ) -> Result<(), Error> {
            let obj_to_send = self.create_method(method, params);
            debug!("Sending method call to socket: {:?}", obj_to_send);
            self.client
                .socket_out
                .send(serde_json::to_string(&obj_to_send)?)?;
            Ok(())
        }

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

}

pub mod chat {

    use super::{connect, ClientSocketWrapper};
    use crate::chat::models::{Event, Method, Reply};
    use atomic_counter::{AtomicCounter, ConsistentCounter};
    use failure::{format_err, Error};
    use log::debug;
    use serde_json::Value;
    use std::{collections::HashMap, convert::TryFrom, sync::mpsc::Receiver};

    pub enum StreamMessage {
        Event(Event),
        Reply(Reply),
    }

    struct ChatClient {
        client: ClientSocketWrapper,
        method_counter: ConsistentCounter,
    }

    impl ChatClient {
        fn connect(
            endpoint: &str,
            auth_key: &str,
            client_id: &str,
        ) -> Result<(Self, Receiver<String>), Error> {
            // TODO what to do with auth_key?
            let (client, receiver) = super::connect(endpoint, client_id)?;
            Ok((
                ChatClient {
                    client,
                    method_counter: ConsistentCounter::new(0),
                },
                receiver,
            ))
        }

        pub fn create_method(
            &mut self,
            method: &str,
            arguments: &HashMap<String, Value>,
        ) -> Method {
            Method {
                method_type: "method".to_owned(),
                method: method.to_owned(),
                arguments: arguments.clone(),
                id: self.method_counter.inc(),
            }
        }

        pub fn call_method(
            &mut self,
            method: &str,
            arguments: &HashMap<String, Value>,
        ) -> Result<(), Error> {
            let obj_to_send = self.create_method(method, arguments);
            debug!("Sending method call to socket: {:?}", obj_to_send);
            self.client
                .socket_out
                .send(serde_json::to_string(&obj_to_send)?)?;
            Ok(())
        }

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

}
