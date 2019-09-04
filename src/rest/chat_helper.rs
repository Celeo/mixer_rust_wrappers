//! Helper for chat-related REST API endpoints.

use super::REST;
use failure::Error;
use log::debug;

/// Helper for chat-related REST API endpoints.
pub struct ChatHelper<'a> {
    /// Reference to constructing REST struct
    pub rest: &'a REST,
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
    /// # use mixer_wrappers::rest::REST;
    /// # let api = REST::new("");
    /// let helper = api.chat_helper();
    /// let channel_id = helper.get_channel_id("some_username");
    /// ```
    pub fn get_channel_id(&self, username: &str) -> Result<usize, Error> {
        debug!("Getting channel id for username {}", username);
        let text = self.rest.query(
            "GET",
            &format!("channels/{}?fields=id", username),
            None,
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
    /// # use mixer_wrappers::rest::REST;
    /// # let api = REST::new("");
    /// let helper = api.chat_helper();
    /// let servers = helper.get_servers(1234567890);
    /// ```
    pub fn get_servers(&self, channel_id: usize) -> Result<Vec<String>, Error> {
        debug!("Getting servers for channel ID {}", channel_id);
        let text = self
            .rest
            .query("GET", &format!("chats/{}", channel_id), None, None, None)?;
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

    #[test]
    fn test_get_channel_id() {
        let _m1 = mock("GET", "/channels/aaaaaa?fields=id")
            .with_body(r#"{"id":123}"#)
            .create();
        let rest = REST::new("");
        let helper = rest.chat_helper();
        let id = helper.get_channel_id("aaaaaa").unwrap();
        assert_eq!(123, id);
    }

    #[test]
    fn test_get_servers() {
        let _m1 = mock("GET", "/chats/123")
            .with_body(r#"{"endpoints":["a","b","c"]}"#)
            .create();
        let rest = REST::new("");
        let helper = rest.chat_helper();
        let servers = helper.get_servers(123).unwrap();
        assert_eq!(vec!["a", "b", "c"], servers);
    }
}
