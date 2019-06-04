use diesel::{r2d2, Connection};

pub trait WundergraphContext {
    type Connection: Connection + 'static;
    fn get_connection(&self) -> &Self::Connection;
}

impl<Conn> WundergraphContext for r2d2::PooledConnection<r2d2::ConnectionManager<Conn>>
where
    Conn: Connection + 'static,
    Self: Connection<Backend = Conn::Backend>,
{
    type Connection = Self;

    fn get_connection(&self) -> &Self {
        self
    }
}
