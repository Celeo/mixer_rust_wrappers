use failure::Error;
use reqwest::{
    header::{HeaderMap, HeaderName, HeaderValue},
    Client, Method,
};
use serde::Serialize;
use std::time::Duration;

use super::errors::BadHttpResponseError;

const BASE_URL: &str = "https://mixer.com/api/v1/";
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
    /// use mixer_rust_wrappers::rest::REST;
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
    /// This is the _raw_ way to query the API; it's most often best
    /// to make use of one of the many wrapper functions to simplify
    /// the casting of JSON to structs.
    ///
    /// # Arguments
    ///
    /// * `method` - HTTP verb
    /// * `endpoint` - API endpoint (do not include the API base URL)
    /// * `params` - query params to include (if none, just send `&[]`)
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let text = api.query(Method::GET, "some/endpoint", &[]).unwrap();
    /// ```
    pub fn query<T: Serialize + ?Sized>(
        &self,
        method: Method,
        endpoint: &str,
        params: &T,
    ) -> Result<String, Error> {
        let endpoint = format!("{}{}", BASE_URL, endpoint);
        let req = self
            .client
            .request(method, &endpoint)
            .headers(self.headers())
            .query(params)
            .build()?;
        let mut resp = self.client.execute(req)?;
        if !resp.status().is_success() {
            return Err(BadHttpResponseError(resp.status().as_u16()).into());
        }
        let text = resp.text()?;
        Ok(text)
    }
}
