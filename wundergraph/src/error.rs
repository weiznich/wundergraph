//! This module contains all error handling related functionality in wundergraph

use crate::scalar::WundergraphScalarValue;
use thiserror::Error;

/// The main error type of wundergraph
#[derive(Debug, Error)]
pub enum WundergraphError {
    /// Indicates that it was not possible to build a filter from the given
    /// graphql arguments
    #[error("Could not build filter from arguments")]
    CouldNotBuildFilterArgument,
    /// Indicates that a unknown database field name was passed into
    /// wundergraph
    #[error("Requested unkown field {name}")]
    UnknownDatabaseField {
        ///The name of the unknown database field
        name: String,
    },
    /// Indicates that a primary key filter could not be build from the
    /// given arguments
    #[error("Could not build primary key filter from arguments")]
    NoPrimaryKeyArgumentFound,
    /// Indicates that building a graphql return value failed
    #[error("Failed to build a return value")]
    JuniperError {
        /// Error returned from juniper
        inner: juniper::FieldError<WundergraphScalarValue>,
    },
    /// Indicates that executing a database query failed
    #[error("Failed to execute query")]
    DieselError {
        /// Error returned from diesel
        #[from]
        inner: diesel::result::Error,
    },
}

/// Commonly used result type
pub type Result<T> = std::result::Result<T, WundergraphError>;
