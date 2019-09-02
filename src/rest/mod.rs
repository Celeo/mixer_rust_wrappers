//! Wrapper around Mixer's REST API.
//!
//! This module contains a struct, `REST` that is contains various helper
//! functions for making calls out to the API and processing the responses.
//!
//! The `ChatHelper` struct can be constructed through an instance of the `REST` struct,
//! providing several handy methods for getting information about the chat server endpoint(s),
//! required for [connecting to chat].
//!
//! [connecting to chat]: ../chat/struct.ChatClient.html#method.connect

/// Error handling
pub mod errors;

use failure::Error;
use reqwest::{
    header::{HeaderMap, HeaderName, HeaderValue},
    Client, Method,
};
use std::time::Duration;

use errors::BadHttpResponseError;

const TIMEOUT: u64 = 10;

/// API wrapper around the Mixer REST API.
pub struct REST {
    client: Client,
    client_id: String,
}

impl REST {
    /// Create a new API wrapper.
    ///
    /// # Arguments
    ///
    /// * `client_id` - your Mixer API client ID
    ///
    /// # Examples
    ///
    /// ```rust
    /// use mixer_wrappers::rest::REST;
    ///
    /// let api = REST::new("abcd");
    /// ```
    pub fn new(client_id: &str) -> Self {
        REST {
            client: Client::builder()
                .timeout(Duration::from_secs(TIMEOUT))
                .build()
                .unwrap(),
            client_id: client_id.to_string(),
        }
    }

    /// Get the base REST API URL.
    fn base_url(&self) -> String {
        #[cfg(not(test))]
        return "https://mixer.com/api/v1".to_owned();
        #[cfg(test)]
        return mockito::server_url();
    }

    /// Build the required API headers.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let headers = self.headers();
    /// ```
    fn headers(&self) -> HeaderMap {
        let mut map = HeaderMap::new();
        map.insert(
            HeaderName::from_static("client-id"),
            HeaderValue::from_bytes(self.client_id.as_bytes()).unwrap(),
        );
        map
    }

    /// Query an endpoint.
    ///
    /// # Arguments
    ///
    /// * `method` - HTTP verb
    /// * `endpoint` - API endpoint (do not include the API base URL)
    /// * `params` - query params to include (if none, just send `&[]`)
    /// * `body` - optional HTTP body String
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use mixer_wrappers::REST;
    /// # use reqwest::Method;
    /// let api = REST::new("");
    /// let text = api.query(Method::GET, "some/endpoint", None, None).unwrap();
    /// ```
    pub fn query(
        &self,
        method: Method,
        endpoint: &str,
        params: Option<&[(&str, &str)]>,
        body: Option<&str>,
    ) -> Result<String, Error> {
        let endpoint = format!("{}/{}", self.base_url(), endpoint);
        let mut builder = self
            .client
            .request(method, &endpoint)
            .headers(self.headers());
        if params.is_some() {
            builder = builder.query(params.unwrap());
        }
        if body.is_some() {
            builder = builder.body(body.unwrap().to_owned());
        }
        let req = builder.build()?;
        let mut resp = self.client.execute(req)?;
        if !resp.status().is_success() {
            return Err(BadHttpResponseError(resp.status().as_u16()).into());
        }
        let text = resp.text()?;
        Ok(text)
    }

    /// Get a struct with several chat-related endpoint helpers.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use mixer_wrappers::REST;
    /// # use reqwest::Method;
    /// let api = REST::new("");
    /// let helper = api.chat_helper();
    /// ```
    pub fn chat_helper(&self) -> ChatHelper {
        ChatHelper { rest: self }
    }
}

/// Helper for chat-related REST API endpoints.
pub struct ChatHelper<'a> {
    rest: &'a REST,
}

impl<'a> ChatHelper<'a> {
    /// Get the channel ID for a username.
    ///
    /// See docs for more information: https://dev.mixer.com/reference/chat/connection#connection
    ///
    /// # Arguments
    ///
    /// * `username` - username to look up
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use mixer_wrappers::rest::{ChatHelper, REST};
    /// # let api = REST::new("");
    /// let helper = api.chat_helper();
    /// let channel_id = helper.get_channel_id("some_username");
    /// ```
    pub fn get_channel_id(&self, username: &str) -> Result<usize, Error> {
        let text = self.rest.query(
            Method::GET,
            &format!("channels/{}?fields=id", username),
            None,
            None,
        )?;
        let json: serde_json::Value = serde_json::from_str(&text)?;
        let channel_id = json["id"].as_u64().unwrap() as usize;
        Ok(channel_id)
    }

    /// Gets a list of chat servers to connect to for the channel ID.
    ///
    /// See docs for more information: https://dev.mixer.com/reference/chat/connection#connection
    ///
    /// # Arguments
    ///
    /// * `channel_id` - channel ID to connect to
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use mixer_wrappers::rest::{ChatHelper, REST};
    /// # let api = REST::new("");
    /// let helper = api.chat_helper();
    /// let servers = helper.get_servers(1234567890);
    /// ```
    pub fn get_servers(&self, channel_id: usize) -> Result<Vec<String>, Error> {
        let text = self
            .rest
            .query(Method::GET, &format!("chats/{}", channel_id), None, None)?;
        let json: serde_json::Value = serde_json::from_str(&text)?;
        let endpoints: Vec<String> = json["endpoints"]
            .as_array()
            .unwrap()
            .iter()
            .map(|e| e.as_str().unwrap().to_owned())
            .collect();
        Ok(endpoints)
    }
}

#[cfg(test)]
mod tests {
    use super::REST;
    use mockito::mock;
    use reqwest::Method;

    #[test]
    fn headers() {
        let rest = REST::new("foobar");
        let headers = rest.headers();
        assert_eq!(1, headers.len());
        assert_eq!(
            "foobar",
            headers.get("client-id").unwrap().to_str().unwrap()
        );
    }

    #[test]
    fn query_good() {
        let body = "hello world";
        let _m1 = mock("GET", "/somewhere?foo=bar").with_body(body).create();
        let rest = REST::new("");
        let resp = rest
            .query(
                Method::GET,
                "somewhere",
                Some(&[("foo", "bar")]),
                Some("hello world"),
            )
            .unwrap();
        assert_eq!(body, resp);
    }

    #[test]
    fn query_wrong_status() {
        let body = "hello world";
        let _m1 = mock("GET", "/somewhere?hello=world")
            .with_body(body)
            .create();
        let rest = REST::new("");
        let resp = rest.query(Method::GET, "somewhere", Some(&[("foo", "bar")]), None);
        assert_eq!(true, resp.is_err());
        let _ = resp.unwrap_err();
    }
}
