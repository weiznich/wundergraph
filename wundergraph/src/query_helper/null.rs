use diesel::backend::Backend;
use diesel::expression::{AppearsOnTable, Expression, NonAggregate, SelectableExpression};
use diesel::query_builder::{AstPass, QueryFragment, QueryId};
use diesel::result::QueryResult;
use diesel::sql_types::{NotNull, Nullable};
use std::marker::PhantomData;

#[derive(Debug, Clone, Copy)]
pub struct Null<ST>(PhantomData<ST>);

pub fn null<ST>() -> Null<ST> {
    Null(PhantomData)
}

impl<ST> Expression for Null<ST>
where
    ST: NotNull,
{
    type SqlType = Nullable<ST>;
}

impl<ST, DB> QueryFragment<DB> for Null<ST>
where
    DB: Backend,
{
    fn walk_ast(&self, mut pass: AstPass<'_, DB>) -> QueryResult<()> {
        pass.push_sql(" NULL ");
        Ok(())
    }
}

impl<ST> QueryId for Null<ST>
where
    ST: QueryId + NotNull,
{
    type QueryId = <Nullable<ST> as QueryId>::QueryId;
    const HAS_STATIC_QUERY_ID: bool = true;
}

impl<ST> NonAggregate for Null<ST> {}

impl<ST, QS> AppearsOnTable<QS> for Null<ST> where Self: Expression {}

impl<T, ST> SelectableExpression<T> for Null<ST> where Self: Expression {}
