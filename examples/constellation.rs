use failure::Error;
use log::{debug, info};
use mixer_wrappers::{ConstellationClient, REST};
use serde_json::Value;
use std::{thread, time::Duration};

const USERNAME: &str = "YOUR_USERNAME";

fn get_client_id() -> String {
    std::fs::read_to_string("client_id")
        .unwrap()
        .trim()
        .to_owned()
}

fn get_channel_id(client_id: &str, username: &str) -> Result<usize, Error> {
    let rest = REST::new(client_id);
    let text = rest.query(
        reqwest::Method::GET,
        &format!("channels/{}", username),
        Some(&[("fields", "id")]),
        None,
    )?;
    let json: Value = serde_json::from_str(&text)?;
    let id = json["id"].as_u64().unwrap() as usize;
    debug!("Channel id for username '{}' is {}", USERNAME, id);
    Ok(id)
}

fn main() {
    log4rs::init_file("log4rs.yml", Default::default()).unwrap();
    let client_id = get_client_id();
    let channel_id = get_channel_id(&client_id, USERNAME).unwrap();

    let (mut client, receiver) = ConstellationClient::connect(&client_id).unwrap();
    let read_handler = thread::spawn(move || loop {
        if let Ok(msg) = receiver.try_recv() {
            info!(">> {}", msg);
        }
    });

    thread::sleep(Duration::from_secs(3));

    client
        .subscribe(&[&format!("channel:{}:update", channel_id)])
        .unwrap();

    read_handler.join().unwrap();
}
