
use diesel::backend::Backend;
use diesel::expression::{AppearsOnTable, Expression, NonAggregate, SelectableExpression};
use diesel::query_builder::{AstPass, QueryFragment, QueryId};
use diesel::result::QueryResult;
use diesel::sql_types::IntoNullable;

pub trait BoxableFilter<QS, DB>
where
    DB: Backend,
    Self: Expression,
    Self: AppearsOnTable<QS>,
    Self: NonAggregate,
    Self: QueryFragment<DB>,
{
}

impl<QS, T, DB> BoxableFilter<QS, DB> for T
where
    DB: Backend,
    T: Expression,
    T: AppearsOnTable<QS>,
    T: NonAggregate,
    T: QueryFragment<DB>,
{
}

#[derive(Debug)]
pub enum MaybeNull<T> {
    Expr(T),
    Null,
}

impl<T> Expression for MaybeNull<T>
where
    T: Expression,
    T::SqlType: IntoNullable,
{
    type SqlType = <T::SqlType as IntoNullable>::Nullable;
}

impl<T, DB> QueryFragment<DB> for MaybeNull<T>
where
    DB: Backend,
    T: QueryFragment<DB>,
{
    fn walk_ast(&self, mut pass: AstPass<'_, DB>) -> QueryResult<()> {
        match self {
            MaybeNull::Expr(e) => e.walk_ast(pass)?,
            MaybeNull::Null => pass.push_sql(" NULL "),
        }
        Ok(())
    }
}

impl<ST> QueryId for MaybeNull<ST>
where
    ST: QueryId,
{
    type QueryId = ();
    const HAS_STATIC_QUERY_ID: bool = false;
}

impl<T> NonAggregate for MaybeNull<T> {}

impl<T, QS> AppearsOnTable<QS> for MaybeNull<T> where Self: Expression {}

impl<T, ST> SelectableExpression<T> for MaybeNull<ST> where Self: Expression {}
