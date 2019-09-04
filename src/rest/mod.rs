//! Wrapper around Mixer's REST API.
//!
//! This module contains a struct, `REST` that is contains various helper
//! functions for making calls out to the API and processing the responses.
//!
//! The `ChatHelper` struct can be constructed through an instance of the `REST` struct,
//! providing several handy methods for getting information about the chat server endpoint(s),
//! required for [connecting to chat].
//!
//! The `WebHookHelper` struct can be constructed through an instance of the `REST` struct,
//! providing several handy methods for registering webhooks, as the HTTP call to do so
//! differs from the rest of the API endpoints.
//!
//! Some endpoints require OAuth. You can utilize this crate's [oauth module] for getting
//! an access token from users.
//!
//! [connecting to chat]: ../chat/struct.ChatClient.html#method.connect
//! [oauth module]: ../oauth

pub mod chat_helper;
pub mod errors;
pub mod webhook_helper;

use failure::Error;
use log::debug;
use reqwest::{
    header::{self, HeaderMap, HeaderName, HeaderValue},
    Client, Method,
};
use std::time::Duration;

use chat_helper::ChatHelper;
use errors::BadHttpResponseError;
use webhook_helper::WebHookHelper;

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
    /// # Arguments
    ///
    /// * `access_token` - optional OAuth token
    fn headers(&self, access_token: Option<&str>) -> HeaderMap {
        let mut map = HeaderMap::new();
        map.insert(
            HeaderName::from_static("client-id"),
            HeaderValue::from_bytes(self.client_id.as_bytes()).unwrap(),
        );
        if access_token.is_some() {
            map.insert(
                header::AUTHORIZATION,
                HeaderValue::from_bytes(format!("Bearer {}", access_token.unwrap()).as_bytes())
                    .unwrap(),
            );
        }
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
    /// * `access_token` - optional OAuth token
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use mixer_wrappers::REST;
    /// let api = REST::new("");
    /// let text = api.query("GET", "some/endpoint", None, None, None).unwrap();
    /// ```
    pub fn query(
        &self,
        method: &str,
        endpoint: &str,
        params: Option<&[(&str, &str)]>,
        body: Option<&str>,
        access_token: Option<&str>,
    ) -> Result<String, Error> {
        let url = format!("{}/{}", self.base_url(), endpoint);
        let method = Method::from_bytes(method.to_uppercase().as_bytes())?;
        debug!("Making {} call to {}", method, url);
        let mut builder = self
            .client
            .request(method, &url)
            .headers(self.headers(access_token));
        if params.is_some() {
            builder = builder.query(params.unwrap());
        }
        if body.is_some() {
            builder = builder.body(body.unwrap().to_owned());
        }
        let req = builder.build()?;
        let mut resp = self.client.execute(req)?;
        if !resp.status().is_success() {
            debug!(
                "Got status code {} from endpoint, text: {}",
                resp.status().as_str(),
                resp.text()?
            );
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
    /// let api = REST::new("");
    /// let helper = api.chat_helper();
    /// ```
    pub fn chat_helper(&self) -> ChatHelper {
        ChatHelper { rest: self }
    }

    /// Get a struct with several WebHook-related endpoint helpers.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use mixer_wrappers::REST;
    /// let api = REST::new("");
    /// let helper = api.webhook_helper();
    /// ```
    pub fn webhook_helper(&self) -> WebHookHelper {
        WebHookHelper { rest: self }
    }
}

#[cfg(test)]
mod tests {
    use super::REST;
    use mockito::mock;

    #[test]
    fn headers() {
        let rest = REST::new("foobar");
        let headers = rest.headers(None);
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
                "GET",
                "somewhere",
                Some(&[("foo", "bar")]),
                Some("hello world"),
                Some("the_token"),
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
        let resp = rest.query("GET", "somewhere", Some(&[("foo", "bar")]), None, None);
        assert_eq!(true, resp.is_err());
        let _ = resp.unwrap_err();
    }
}
