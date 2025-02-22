use std::fmt;

#[derive(Debug, Clone)]
pub(crate) struct ParseError;
#[derive(Debug, Clone)]
pub(crate) struct InvalidActionError;

impl std::error::Error for ParseError {}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Could not parse client message")
    }
}

impl std::error::Error for InvalidActionError {}

impl fmt::Display for InvalidActionError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Unknown action provided by client")
    }
}
