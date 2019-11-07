//! A module containing extension traits for various diesel types

use diesel::backend::Backend;
use diesel::expression::{AppearsOnTable, Expression, NonAggregate, SelectableExpression};
use diesel::query_builder::{AstPass, QueryFragment, QueryId};
use diesel::result::QueryResult;
use diesel::sql_types::IntoNullable;

/// A helper trait used when boxing filters
///
/// In Rust you cannot create a trait object with more than one trait.
/// This type has all of the additional traits you would want when using
/// `Box<Expression>` as a single trait object. This type is comparable to
/// diesels `BoxableExpression`, but allows to use non select able expressions,
/// which is mainly useful for constructing filters.
///
/// This is typically used as the return type of a function or as associated
/// types in traits.
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

/// A diesel helper type that indicates if null or some expression selected
#[derive(Debug)]
pub enum MaybeNull<T> {
    /// Select the expression
    Expr(T),
    /// Select a null value
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
