#[derive(Debug, Fail)]
pub enum WundergraphError {
    #[fail(display = "Could not build filter from arguments")]
    CouldNotBuildFilterArgument,
    #[fail(display = "Requested unkown field {}", name)]
    UnknownDatabaseField { name: String },
}
