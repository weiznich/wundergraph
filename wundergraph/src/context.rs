use diesel::Connection;

/// A trait for types that could be used as context types for wundergraph
pub trait WundergraphContext {

    /// The underlying connection type
    type Connection: Connection + 'static;

    /// Get a connection from the context
    fn get_connection(&self) -> &Self::Connection;
}

impl<Conn> WundergraphContext for Conn
where
    Conn: Connection + 'static,
{
    type Connection = Self;

    fn get_connection(&self) -> &Self {
        self
    }
}
