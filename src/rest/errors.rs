use failure::Fail;

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
}
