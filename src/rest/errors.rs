use failure::Fail;

#[derive(Debug, Fail)]
#[fail(display = "An error occurred with error code {}.", _0)]
pub struct BadHttpResponseError(pub u16);
