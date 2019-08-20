use failure::Error;
use reqwest::{
    header::{HeaderMap, HeaderName, HeaderValue},
    Client, Method,
};
use serde::Serialize;
use std::time::Duration;

use super::errors::BadHttpResponseError;

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
    /// This is the _raw_ way to query the API; it's most often best
    /// to make use of one of the many wrapper functions to simplify
    /// the casting of JSON to structs.
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
    /// ```rust,ignore
    /// let text = api.query(Method::GET, "some/endpoint", &[]).unwrap();
    /// ```
    pub fn query<P: Serialize + ?Sized>(
        &self,
        method: Method,
        endpoint: &str,
        params: &P,
        body: Option<&str>,
    ) -> Result<String, Error> {
        let endpoint = format!("{}/{}", self.base_url(), endpoint);
        let builder = self
            .client
            .request(method, &endpoint)
            .headers(self.headers())
            .query(params);
        let req = if body.is_some() {
            builder.body(body.unwrap().to_owned()).build()?
        } else {
            builder.build()?
        };
        let mut resp = self.client.execute(req)?;
        if !resp.status().is_success() {
            return Err(BadHttpResponseError(resp.status().as_u16()).into());
        }
        let text = resp.text()?;
        Ok(text)
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
        let _m1 = mock("GET", "/somewhere?foo=bar")
            .with_status(200)
            .with_body(body)
            .create();
        let rest = REST::new("foobar");
        let resp = rest
            .query(Method::GET, "somewhere", &[("foo", "bar")], None)
            .unwrap();
        assert_eq!(body, resp);
    }

    #[test]
    fn query_wrong_status() {
        let body = "hello world";
        let _m1 = mock("GET", "/somewhere?hello=world")
            .with_status(200)
            .with_body(body)
            .create();
        let rest = REST::new("foobar");
        let resp = rest.query(Method::GET, "somewhere", &[("foo", "bar")], None);
        assert_eq!(true, resp.is_err());
        let err = resp.unwrap_err();
        assert_eq!("BadHttpResponseError(501)", &format!("{:?}", err));
    }
}
