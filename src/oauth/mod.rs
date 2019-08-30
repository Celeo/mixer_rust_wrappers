use oauth2::{Config, Token, TokenError};

// https://dev.mixer.com/reference/oauth/quickdetails
const ENDPOINT_AUTH: &str = "https://mixer.com/oauth/authorize";
const ENDPOINT_TOKEN: &str = "https://mixer.com/api/v1/oauth/token";

/// Create an OAuth2 Config struct instance.
///
/// # Arguments
///
/// * `client_id` - your OAuth application id
/// * `client_secret` - your OAuth application secret
/// * `scopes` - your desired OAuth scopes
/// * `redirect_url` - your application's redirect URL
fn init(client_id: &str, client_secret: &str, scopes: &[&str], redirect_url: &str) -> Config {
    let mut config = Config::new(client_id, client_secret, ENDPOINT_AUTH, ENDPOINT_TOKEN);
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

#[cfg(test)]
mod tests {
    use super::get_authorize_url;

    const CLIENT_ID: &str = "a";
    const CLIENT_SECRET: &str = "b";
    const SCOPES: [&str; 2] = ["c", "d"];
    const REDIRECT_URL: &str = "e";

    #[test]
    fn test_get_authorize_url() {
        let url = get_authorize_url(CLIENT_ID, CLIENT_SECRET, &SCOPES, REDIRECT_URL, false);
        let scopes_str = SCOPES.join("+");
        assert!(url.starts_with(&format!("https://mixer.com/oauth/authorize?client_id={}&scope={}&response_type=code&redirect_uri={}&state=",
            CLIENT_ID, scopes_str, REDIRECT_URL)));
    }
}
