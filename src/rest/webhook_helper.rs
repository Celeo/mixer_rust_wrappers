//! Helper for webhook-related REST API endpoints.

use super::REST;
use failure::Error;
use log::debug;
use reqwest::header::{self, HeaderMap, HeaderName, HeaderValue};
use serde_json::json;

/// Helper for webhook-related REST API endpoints.
pub struct WebHookHelper<'a> {
    /// Reference to constructing REST struct
    pub rest: &'a REST,
}

impl<'a> WebHookHelper<'a> {
    /// Register webhooks.
    ///
    /// See the [documentation] for more information.
    ///
    /// # Arguments
    ///
    /// * `events` - list of events to receive
    /// * `url` - URL to receive the call at
    /// * `client_secret` - your OAuth app's client_secret
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use mixer_wrappers::rest::REST;
    /// # let api = REST::new("");
    /// let helper = api.webhook_helper();
    /// let channel_id = helper.register(&["event_1", "event_2"], "http://example.com/callback", "your_client_secret").unwrap();
    /// ```
    ///
    /// [documentation]: https://dev.mixer.com/reference/webhooks
    pub fn register(&self, events: &[&str], url: &str, client_secret: &str) -> Result<(), Error> {
        // This request has to be constructed explicitly here, as it doesn't share many
        // similarities with the normal API requests, namely the headers.
        debug!(
            "Making webhook register call with events: {}",
            events.join(", ")
        );
        let mut headers = HeaderMap::new();
        headers.insert(
            HeaderName::from_static("client-id"),
            HeaderValue::from_bytes(self.rest.client_id.as_bytes()).unwrap(),
        );
        headers.insert(
            header::AUTHORIZATION,
            HeaderValue::from_bytes(format!("Secret {}", client_secret).as_bytes()).unwrap(),
        );
        let body = json!({
            "events": events,
            "kind": "web",
            "url": url,
        });
        self.rest
            .client
            .post(&format!("{}/hooks", self.rest.base_url()))
            .headers(headers)
            .body(serde_json::to_string(&body).unwrap())
            .send()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::REST;
    use mockito::mock;

    #[test]
    fn test_register() {
        let _m1 = mock("POST", "/hook").create();
        let rest = REST::new("");
        let helper = rest.webhook_helper();
        helper
            .register(
                &["event_1", "event_2"],
                "http://example.com/callback",
                "aaaaaa",
            )
            .unwrap();
    }
}
