//! REST API error handling.

use failure::Fail;

/// Error for receiving a non-20X response from an endpoint.
#[derive(Debug, Fail, PartialEq)]
#[fail(display = "An error occurred with error code {}.", _0)]
pub struct BadHttpResponseError(pub u16);

#[cfg(test)]
mod tests {
    use super::BadHttpResponseError;

    #[test]
    fn has_display() {
        let err = BadHttpResponseError(400);
        let _ = format!("{}", err);
    }

    #[test]
    fn has_partial_eq() {
        let err1 = BadHttpResponseError(400);
        let err2 = BadHttpResponseError(400);

        assert_eq!(err1, err2);
    }
}
