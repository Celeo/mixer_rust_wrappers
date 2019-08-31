use log::{debug, info};
use mixer_wrappers::{ChatClient, REST};
use std::thread;

const USERNAME: &str = "YOUR_USERNAME";

fn get_client_id() -> String {
    std::fs::read_to_string("client_id")
        .unwrap()
        .trim()
        .to_owned()
}

fn main() {
    let client_id = get_client_id();
    let api = REST::new(&client_id);
    let chat_helper = api.chat_helper();
    let channel_id = chat_helper
        .get_channel_id(USERNAME)
        .expect("Couldn't get channel id");
    let endpoints = chat_helper
        .get_servers(channel_id)
        .expect("Couldn't get chat server");
    let (mut client, receiver) =
        ChatClient::connect(&endpoints[0], &client_id).expect("Could not connect to chat");
    debug!("Authenticating");
    client
        .authenticate(channel_id, None, None)
        .expect("Could not authenticate");
    debug!("Connected");
    let receiver_handler = thread::spawn(move || loop {
        if let Ok(msg) = receiver.try_recv() {
            info!(">> {}", msg);
        }
    });
    debug!("Set up receiver reader");
    client.join_handle.join().expect("Could not join thread");
    receiver_handler.join().expect("Could not join thread");
}
