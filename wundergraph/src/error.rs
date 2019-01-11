//! This module contains all error handling related functionality in wundergraph

/// The main error type of wundergraph
#[derive(Debug, Fail)]
pub enum WundergraphError {
    /// Indicates that it was not possible to build a filter from the given
    /// graphql arguments
    #[fail(display = "Could not build filter from arguments")]
    CouldNotBuildFilterArgument,
    /// Indicates that a unknown database field name was passed into
    /// wundergraph
    #[fail(display = "Requested unkown field {}", name)]
    UnknownDatabaseField {
        ///The name of the unknown database field
        name: String,
    },
    #[fail(display = "Could not build primary key filter from arguments")]
    NoPrimaryKeyArgumentFound,
}
