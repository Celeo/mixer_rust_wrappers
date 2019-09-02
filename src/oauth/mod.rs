//! Wrappers around OAuth calls for authenticating the user
//! interacting with your application.
//!
//! This module does not contain any structs; instead, import methods individually as needed.
//!
//! `get_authorize_url` is used to start your application's user on Mixer's standard OAuth flow, where
//! Mixer has the user authenticate and confirm using the application and then redirects them to the
//! configured redirect URL, where a web server needs to be running to take the code from the user
//! and exchange it for the OAuth token.
//!
//! `get_token_from_code` is used for exchanging the code for the token in the normal flow.
//!
//! `get_access_token_from_refresh` is used to get another access token from the refresh token.
//!
//! `get_shortcode` is used for generating a 6-digit code for the application's user to enter on
//! Mixer's "shortcode" OAuth flow, which is useful when the application does not contain a web server
//! to receive the code from the user. This code must be given to the user so that they can enter it
//! on Mixer's site.
//!
//! `check_shortcode` is used to poll the Mixer API for the status of a user entering (or not entering)
//! a shortcode.

use oauth2::{Config, Token, TokenError};
use reqwest::Client;
use serde_derive::Deserialize;
use serde_json::{json, Value};

/// Struct around the response from fetching an auth shortcode.
#[derive(Debug, Deserialize)]
pub struct ShortcodeResponse {
    /// Code that the user being authenticated needs to enter
    pub code: String,
    /// Expiration time in seconds
    pub expires_in: u64,
    /// Handle string to check on the user's authentication status
    pub handle: String,
}

/// Status of a shortcode auth flow.
#[derive(Debug, PartialEq)]
pub enum ShortcodeStatus {
    /// HTTP 204 - user hasn't entered the code yet
    WaitingOnUser,
    /// HTTP 403 - user chose not to authenticate
    UserDeniedAccess,
    /// HTTP 404 - handle is invalid or expired
    HandleInvalid,
    /// HTTP 202 - user completed the authentication
    UserGrantedAccess(String),
}

/// Get the endpoint for authorizing a user.
///
/// https://dev.mixer.com/reference/oauth/quickdetails
fn get_endpoint_auth_url() -> String {
    #[cfg(not(test))]
    return "https://mixer.com/oauth/authorize".to_owned();
    #[cfg(test)]
    return mockito::server_url();
}

/// Get the endpoint for exchanging the code for a token.
///
/// https://dev.mixer.com/reference/oauth/quickdetails
fn get_endpoint_token_url() -> String {
    #[cfg(not(test))]
    return "https://mixer.com/api/v1/oauth/token".to_owned();
    #[cfg(test)]
    return mockito::server_url();
}

/// Get the endpoint for creating a shortcode.
///
/// https://dev.mixer.com/reference/oauth/shortcodeauth#shortcode-flow-specification
fn get_shortcode_url_start() -> String {
    #[cfg(not(test))]
    return "https://mixer.com/api/v1/oauth/shortcode ".to_owned();
    #[cfg(test)]
    return mockito::server_url();
}

/// Get the endpoint for creating a shortcode.
///
/// https://dev.mixer.com/reference/oauth/shortcodeauth#shortcode-flow-specification
///
/// # Arguments
///
/// * `_handle` - handle from the initial shortcode response
fn get_shortcode_url_check(_handle: &str) -> String {
    #[cfg(not(test))]
    return format!(
        "{}{}",
        "https://mixer.com/api/v1/oauth/shortcode/check/", _handle
    );
    #[cfg(test)]
    return mockito::server_url();
}

/// Create an OAuth2 Config struct instance.
///
/// # Arguments
///
/// * `client_id` - your OAuth application id
/// * `client_secret` - your OAuth application secret
/// * `scopes` - your desired OAuth scopes
/// * `redirect_url` - your application's redirect URL
fn init(client_id: &str, client_secret: &str, scopes: &[&str], redirect_url: &str) -> Config {
    let mut config = Config::new(
        client_id,
        client_secret,
        get_endpoint_auth_url(),
        get_endpoint_token_url(),
    );
    for scope in scopes {
        config = config.add_scope((*scope).to_owned());
    }
    config = config.set_redirect_url(redirect_url);
    config = config.set_state(format!("{}", rand::random::<u64>()));
    config
}

/// Get the authorize URL for your application.
///
/// # Arguments
///
/// * `client_id` - your OAuth application id
/// * `client_secret` - your OAuth application secret
/// * `scopes` - your desired OAuth scopes
/// * `redirect_url` - your application's redirect URL
/// * `force` - set to `true` to force re-authentication [doc link]
///
/// # Examples
///
/// ```rust,no_run
/// # use mixer_wrappers::oauth::get_authorize_url;
/// let url = get_authorize_url("aaa", "bbb", &["s_1", "s_2", "s_3"], "ccc", false);
/// ```
///
/// [doc link]: https://dev.mixer.com/reference/oauth#reauthorizing-an-application
pub fn get_authorize_url(
    client_id: &str,
    client_secret: &str,
    scopes: &[&str],
    redirect_url: &str,
    force: bool,
) -> String {
    let config = init(client_id, client_secret, scopes, redirect_url);
    let mut url = config.authorize_url();
    if force {
        url.query_pairs_mut()
            .append_pair("approval_prompt", "force");
    }
    url.into_string()
}

/// Exchange the code from a user's browser for an OAuth token.
///
/// # Arguments
///
/// * `client_id` - your OAuth application id
/// * `client_secret` - your OAuth application secret
/// * `scopes` - your desired OAuth scopes
/// * `redirect_url` - your application's redirect URL
/// * `code` - the code from the user
///
/// # Examples
///
/// ```rust,no_run
/// # use mixer_wrappers::oauth::get_token_from_code;
/// let token = get_token_from_code("aaa", "bbb", &["s_1", "s_2", "s_3"], "ccc", "code_here").unwrap();
/// ```
pub fn get_token_from_code(
    client_id: &str,
    client_secret: &str,
    scopes: &[&str],
    redirect_url: &str,
    code: &str,
) -> Result<Token, TokenError> {
    let config = init(client_id, client_secret, scopes, redirect_url);
    config.exchange_code(code)
}

/// Exchange a refresh token for another access token.
///
/// This is required when the access token from a successful authentication expires -
/// the refresh token is used to make another authentication request to the API to
/// get another access token. Only the access token can be use to interact with the
/// API on the user's behalf.
///
/// # Arguments
///
/// * `client_id` - your OAuth application id
/// * `client_secret` - your OAuth application secret
/// * `scopes` - your desired OAuth scopes
/// * `redirect_url` - your application's redirect URL
/// * `refresh_token` - the refresh token from the successful auth
///
/// # Examples
///
/// ```rust,no_run
/// # use mixer_wrappers::oauth::get_access_token_from_refresh;
/// let new_token = get_access_token_from_refresh("aaa", "bbb", &["s_1", "s_2", "s_3"], "ccc", "refresh_token_here").unwrap();
/// ```
pub fn get_access_token_from_refresh(
    client_id: &str,
    client_secret: &str,
    scopes: &[&str],
    redirect_url: &str,
    refresh_token: &str,
) -> Result<Token, TokenError> {
    let config = init(client_id, client_secret, scopes, redirect_url);
    config.exchange_refresh_token(refresh_token)
}

/// Get an authentication shortcode.
///
/// This is used for completing the OAuth flow for a user without supplying a redirect URL
/// for them to land on after authenticating, or when "for scenarios where it is difficult to
/// embed a browser or require the user to give keyboard input" [docs].
///
/// Once the application receives the response, the `code` field needs to be given to the user,
/// who needs to enter it into https://mixer.com/go. The application needs to monitor whether
/// or not the user has done so via making repeated API calls to another method, wrapped in
/// this library by the `check_shortcode` function.
///
/// # Arguments
///
/// * `client_id` - your OAuth application id
/// * `client_secret` - your OAuth application secret
/// * `scopes` - your desired OAuth scopes
///
/// # Examples
///
/// ```rust,no_run
/// # use mixer_wrappers::oauth::get_shortcode;
/// let shortcode = get_shortcode("aaa", "bbb", &["s_1", "s_2", "s_3"]).unwrap();
/// ```
///
/// [docs]: https://dev.mixer.com/reference/oauth/shortcodeauth
pub fn get_shortcode(
    client_id: &str,
    client_secret: &str,
    scopes: &[&str],
) -> Result<ShortcodeResponse, failure::Error> {
    let client = Client::new();
    let json = json!({
        "client_id": client_id,
        "client_secret": client_secret,
        "scope": scopes.join(" "),
    });
    let mut resp = client.post(&get_shortcode_url_start()).json(&json).send()?;
    let data: ShortcodeResponse = resp.json()?;
    Ok(data)
}

/// Check on the status of a shortcode.
///
/// This method returns an enum value representing the current state of the code
/// by making a single call to the Mixer API with the handle.
///
/// Application authors will need to call this method repeatedly, waiting on the
/// user to visit the site, enter the code, and confirm authentication. This is
/// intended to be done with threads, but if your application *must* wait for the
/// user to complete the authentication flow before proceeding, it can just loop
/// calling and sleeping.
///
/// # Arguments
///
/// * `handle` - the handle received from starting the shortcode flow; this
///              is not the code that's sent to the user
///
/// # Examples
///
/// ```rust,no_run
/// # use mixer_wrappers::oauth::{check_shortcode, ShortcodeStatus};
/// # use std::{thread, time::Duration};
/// loop {
///     let status = check_shortcode("some_handle");
///     let code: String = match status {
///         ShortcodeStatus::UserGrantedAccess(ref c) => c.to_owned(),
///         ShortcodeStatus::UserDeniedAccess => break,
///         ShortcodeStatus::HandleInvalid => break,
///         _ => {
///             thread::sleep(Duration::from_secs(3));
///             continue;
///         }
///     };
///     break;
/// }
/// ```
pub fn check_shortcode(handle: &str) -> ShortcodeStatus {
    let mut resp = match reqwest::get(&get_shortcode_url_check(handle)) {
        Ok(r) => r,
        Err(_) => return ShortcodeStatus::HandleInvalid,
    };
    match resp.status().as_u16() {
        200 => {
            let data: Value = resp.json().unwrap();
            let code = data["code"].as_str().unwrap();
            ShortcodeStatus::UserGrantedAccess(code.to_owned())
        }
        204 => ShortcodeStatus::WaitingOnUser,
        403 => ShortcodeStatus::UserDeniedAccess,
        _ => ShortcodeStatus::HandleInvalid,
    }
}

#[cfg(test)]
mod tests {
    use super::{
        check_shortcode, get_access_token_from_refresh, get_authorize_url, get_shortcode,
        get_token_from_code, ShortcodeStatus,
    };
    use mockito::mock;

    const CLIENT_ID: &str = "a";
    const CLIENT_SECRET: &str = "b";
    const SCOPES: [&str; 2] = ["c", "d"];
    const REDIRECT_URL: &str = "e";

    #[test]
    fn test_get_authorize_url() {
        let url = get_authorize_url(CLIENT_ID, CLIENT_SECRET, &SCOPES, REDIRECT_URL, false);
        let scopes_str = SCOPES.join("+");
        assert!(!url.contains("approval_prompt=force"));
        assert!(url.contains(&format!(
            "?client_id={}&scope={}&response_type=code&redirect_uri={}&state=",
            CLIENT_ID, scopes_str, REDIRECT_URL
        )));
    }

    #[test]
    fn test_get_authorize_url_force() {
        let url = get_authorize_url(CLIENT_ID, CLIENT_SECRET, &SCOPES, REDIRECT_URL, true);
        assert!(url.contains("approval_prompt=force"));
    }

    #[test]
    fn test_get_token_from_code() {
        let body = r#"{
            "access_token": "123abc",
            "expires_in": 3600,
            "token_type": "test"
        }"#;
        let _m1 = mock("POST", "/")
            .with_body(body)
            .with_header("Content-Type", "application/json")
            .create();
        let token =
            get_token_from_code(CLIENT_ID, CLIENT_SECRET, &SCOPES, REDIRECT_URL, "123abc").unwrap();
        assert_eq!("123abc", token.access_token);
    }

    #[test]
    fn test_get_access_token_from_refresh() {
        let body = r#"{
            "access_token": "123abc",
            "expires_in": 3600,
            "token_type": "test"
        }"#;
        let _m1 = mock("POST", "/")
            .with_body(body)
            .with_header("Content-Type", "application/json")
            .create();
        let token = get_access_token_from_refresh(
            CLIENT_ID,
            CLIENT_SECRET,
            &SCOPES,
            REDIRECT_URL,
            "123abc",
        )
        .unwrap();
        assert_eq!("123abc", token.access_token);
    }

    #[test]
    fn test_get_shortcode() {
        let body = r#"{
            "code": "foo",
            "expires_in": 120,
            "handle": "bar"
        }"#;
        let _m1 = mock("POST", "/")
            .with_header("Content-Type", "application/json")
            .with_body(body)
            .create();
        let response = get_shortcode(CLIENT_ID, CLIENT_SECRET, &SCOPES).unwrap();
        assert_eq!("foo", response.code);
        assert_eq!(120, response.expires_in);
        assert_eq!("bar", response.handle);
    }

    #[test]
    fn test_check_shortcode_200() {
        let body = r#"{"code": "foo"}"#;
        let _m1 = mock("GET", "/")
            .with_header("Content-Type", "application/json")
            .with_body(body)
            .create();
        let status = check_shortcode("bar");
        assert_eq!(status, ShortcodeStatus::UserGrantedAccess("foo".to_owned()));
    }

    #[test]
    fn test_check_shortcode_204() {
        let _m1 = mock("GET", "/").with_status(204).create();
        let status = check_shortcode("bar");
        assert_eq!(status, ShortcodeStatus::WaitingOnUser);
    }

    #[test]
    fn test_check_shortcode_403() {
        let _m1 = mock("GET", "/").with_status(403).create();
        let status = check_shortcode("bar");
        assert_eq!(status, ShortcodeStatus::UserDeniedAccess);
    }

    #[test]
    fn test_check_shortcode_404() {
        let _m1 = mock("GET", "/").with_status(404).create();
        let status = check_shortcode("bar");
        assert_eq!(status, ShortcodeStatus::HandleInvalid);
    }
}
