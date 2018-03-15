use std::fmt::{self, Display};
use std::error;

#[derive(Debug)]
pub enum Error {
    DieselError(::diesel::result::Error),
    CouldNotBuildFilterArgument,
    UnknownDatabaseField(String),
    Other(Box<error::Error + Send + Sync>),
}

impl From<::diesel::result::Error> for Error {
    fn from(e: ::diesel::result::Error) -> Self {
        Error::DieselError(e)
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "An error occured: {:?}", self)
    }
}
