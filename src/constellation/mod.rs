/// Error handling
pub mod errors;
/// Static models for the JSON data
pub mod models;

use atomic_counter::{AtomicCounter, ConsistentCounter};
use failure::{format_err, Error};
use log::{debug, error, info, warn};
use serde_json::Value;
use std::{
    collections::HashMap,
    convert::TryFrom,
    sync::mpsc::{channel, Receiver, Sender as ChanSender},
    thread::{self, JoinHandle},
};
use url::Url;
use ws::{
    connect as socket_connect, CloseCode, Error as SocketError, Handler, Handshake,
    Message as SocketMessage, Request, Result as WSResult, Sender as SocketSender,
};

use models::{Event, Method, Reply, StreamMessage};

struct SocketClient {
    client_id: String,
    connection_sender: ChanSender<bool>,
    message_sender: ChanSender<StreamMessage>,
}

impl SocketClient {
    /// Create a new low-level client.
    fn new(
        client_id: &str,
        connection_sender: ChanSender<bool>,
        message_sender: ChanSender<StreamMessage>,
    ) -> Self {
        SocketClient {
            client_id: client_id.to_owned(),
            connection_sender,
            message_sender,
        }
    }
}

impl Handler for SocketClient {
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
            let as_text = msg.as_text().unwrap();
            let json: serde_json::Value = match serde_json::from_str(&as_text) {
                Ok(j) => j,
                Err(e) => {
                    error!("Could not parse JSON: {}", e);
                    return Ok(());
                }
            };
            let type_ = match json["type"].as_str() {
                Some(t) => t,
                None => {
                    error!("Message does not have 'type' field");
                    return Ok(());
                }
            };
            let event = if type_ == "event" {
                Some(Event::try_from(json.clone()).unwrap())
            } else {
                None
            };
            let reply = if type_ == "reply" {
                Some(Reply::try_from(json).unwrap())
            } else {
                None
            };
            self.message_sender
                .send(StreamMessage { event, reply })
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
pub struct ConstellationClient {
    socket_out: SocketSender,
    connection_receiver: Receiver<bool>,
    /// Thread handle that you can join to to keep your program running
    pub client_thread_handler: JoinHandle<()>,
    is_connected: bool,
    method_counter: ConsistentCounter,
}

impl ConstellationClient {
    /// Create a new high-level client.
    fn new(
        socket_out: SocketSender,
        connection_receiver: Receiver<bool>,
        client_thread_handler: JoinHandle<()>,
    ) -> Self {
        ConstellationClient {
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

    /// Create a new method to send to the socket.
    ///
    /// Handles setting the id field with a unique number.
    ///
    /// # Arguments
    ///
    /// * `method` - which method to call
    /// * `params` - params to include
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let method = client.create_method("some-method-name", &params);
    /// ```
    pub fn create_method(&mut self, method: &str, params: &HashMap<String, Value>) -> Method {
        Method {
            method_type: "method".to_owned(),
            method: method.to_owned(),
            params: params.clone(),
            id: self.method_counter.inc(),
        }
    }

    /// Call a method by sending a method JSON through the socket.
    ///
    /// Responses to the message come asynchronously through the
    /// MPSC Receiver created as part of setting up the client.
    ///
    /// # Arguments
    ///
    /// * `method` - which method to call
    /// * `params` - params to include
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// client.call_method("some-method-name", &HashMap::new()).unwrap();
    /// ```
    pub fn call_method(
        &mut self,
        method: &str,
        params: &HashMap<String, Value>,
    ) -> Result<(), Error> {
        let obj_to_send = self.create_method(method, params);
        debug!("Sending method call to socket: {}", obj_to_send);
        self.socket_out.send(serde_json::to_string(&obj_to_send)?)?;
        Ok(())
    }
}

/// Create a connection to the Mixer Constellation websocket endpoint.
///
/// Returns a tuple of the client you can use to send data to Constellation,
/// and an MPSC Receiver used for getting data out of the socket. This method
/// utilizes threads so that it does not block; the program can continue
/// running after calling this method.
///
/// Of the tuple that's returned, the first struct is the client that is
/// used to send messages to Constellation. The second item is the MPSC
/// receiver that is sent the replies and events back from the socket.
/// Handling these structs is a task for the program.
///
/// # Arguments
///
/// * `client_id` - your Mixer API client ID
///
/// # Examples
///
/// ## Simple method call
///
/// ```rust,no_run
/// # use mixer_wrappers::constellation::connect;
/// let (client, receiver) = connect("aaaaaaaaaa").unwrap();
/// ```
///
/// ## Full program
///
/// ```rust,no_run
/// use mixer_wrappers::constellation::{connect, models::StreamMessage};
/// use std::{collections::HashMap, sync::mpsc::Receiver, thread};
///
/// fn message_handler(receiver: Receiver<StreamMessage>) {
///     loop {
///         if let Ok(message) = receiver.try_recv() {
///             if let Some(event) = message.event {
///                 // ...
///             }
///             if let Some(reply) = message.reply {
///                 // ...
///             }
///         }
///     }
/// }
///
/// fn main() {
///     let (mut client, receiver) = connect("aaaaaaaaaa").expect("Could not start client");
///     thread::spawn(move || {
///         message_handler(receiver);
///     });
///     client.call_method("ping", &HashMap::new()).unwrap();
///     client.client_thread_handler.join().unwrap();
/// }
/// ```
pub fn connect(client_id: &str) -> Result<(ConstellationClient, Receiver<StreamMessage>), Error> {
    debug!("Setting up connection");
    // create channels
    let (ws_send, ws_recv) = channel::<SocketSender>();
    let (conn_send, conn_recv) = channel::<bool>();
    let (msg_send, msg_rev) = channel::<StreamMessage>();

    // launch the socket connection in a new thread
    let client_id = client_id.to_owned();
    let client_handler = thread::spawn(move || {
        debug!("Starting connection");
        socket_connect("wss://constellation.mixer.com", |socket_out| {
            let client = SocketClient::new(&client_id, conn_send.clone(), msg_send.clone());
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
    let client = ConstellationClient::new(socket_out, conn_recv, client_handler);

    // return the final client
    debug!("Connection setup finished");
    Ok((client, msg_rev))
}
