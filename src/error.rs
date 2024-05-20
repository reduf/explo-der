use std::fmt;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Error(pub &'static str);

pub type Result<T> = std::result::Result<T, Error>;

impl From<&'static str> for Error {
    fn from(value: &'static str) -> Error {
        return Self(value);
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        return write!(f, "{}", self.0);
    }
}
