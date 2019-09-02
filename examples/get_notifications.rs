use failure::{format_err, Error};
use mixer_wrappers::{
    oauth::{check_shortcode, get_shortcode, get_token_from_code, ShortcodeStatus},
    REST,
};
use serde_json::Value;
use std::{thread, time::Duration};

const USERNAME: &str = "YOUR_USERNAME";
const CLIENT_ID: &str = "YOUR_CLIENT_ID";
const CLIENT_SECRET: &str = "CLIENT_SECRET";

fn get_access_token() -> Result<String, Error> {
    let resp = get_shortcode(CLIENT_ID, CLIENT_SECRET, &["user:notification:self"]).unwrap();
    println!("Code: {}, go to https://mixer.com/go to enter", resp.code);
    let code: String;
    loop {
        let status = check_shortcode(&resp.handle);
        let c = match status {
            ShortcodeStatus::UserGrantedAccess(ref c) => c.to_owned(),
            ShortcodeStatus::UserDeniedAccess => return Err(format_err!("UserDeniedAccess")),
            ShortcodeStatus::HandleInvalid => return Err(format_err!("HandleInvalid")),
            _ => {
                thread::sleep(Duration::from_secs(5));
                continue;
            }
        };
        code = c;
        break;
    }
    let token = get_token_from_code(
        CLIENT_ID,
        CLIENT_SECRET,
        &["user:notification:self"],
        "",
        &code,
    )
    .unwrap();
    Ok(token.access_token)
}

fn get_user_id(rest: &REST) -> Result<u64, Error> {
    let text = rest.query(
        "GET",
        "users/search",
        Some(&[("query", USERNAME), ("noCount", "true"), ("fields", "id")]),
        None,
        None,
    )?;
    let json: Value = serde_json::from_str(&text)?;
    let id = json.as_array().unwrap()[0]["id"].as_u64().unwrap();
    Ok(id)
}

fn main() {
    let token = get_access_token().unwrap();
    let rest = REST::new(CLIENT_ID);
    let resp = rest
        .query(
            "GET",
            &format!("users/{}/notifications", get_user_id(&rest).unwrap()),
            Some(&[("limit", "5"), ("noCount", "true")]),
            None,
            Some(&token),
        )
        .unwrap();
    println!("{}", resp);
}
