use super::{BoxedQuery, LoadingHandler};
use crate::context::WundergraphContext;
use crate::query_builder::selection::offset::ApplyOffset;
use crate::scalar::WundergraphScalarValue;
use diesel::backend::Backend;
use diesel::query_builder::QueryFragment;
use diesel::{r2d2, Connection, QuerySource};
use failure::Error;
use juniper::LookAheadSelection;

pub trait QueryModifier<L, DB>: WundergraphContext + Sized
where
    L: LoadingHandler<DB, Self>,
    DB: Backend + ApplyOffset + 'static,
{
    fn modify_query<'a>(
        &self,
        select: &LookAheadSelection<'_, WundergraphScalarValue>,
        query: BoxedQuery<'a, L, DB, Self>,
    ) -> Result<BoxedQuery<'a, L, DB, Self>, Error>;
}

impl<Conn, DB, T> QueryModifier<T, DB> for r2d2::PooledConnection<r2d2::ConnectionManager<Conn>>
where
    T: LoadingHandler<DB, Self>,
    Conn: Connection<Backend = DB> + 'static,
    Self: Connection<Backend = DB> + 'static,
    DB: Backend + ApplyOffset + 'static,
    T::Table: 'static,
    <T::Table as QuerySource>::FromClause: QueryFragment<DB>,
    DB::QueryBuilder: Default,
{
    fn modify_query<'a>(
        &self,
        _select: &LookAheadSelection<'_, WundergraphScalarValue>,
        query: BoxedQuery<'a, T, DB, Self>,
    ) -> Result<BoxedQuery<'a, T, DB, Self>, Error> {
        Ok(query)
    }
}
