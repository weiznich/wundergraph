//! A module containing extension traits for various diesel types

use diesel::expression::{AppearsOnTable, Expression, NonAggregate, SelectableExpression};
use diesel::query_builder::{AstPass, QueryFragment};
use diesel::result::QueryResult;
use diesel::sql_types::IntoNullable;
use diesel::{backend::Backend, Column};
use std::marker::PhantomData;

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
pub struct MaybeNull<T> {
    expr: PhantomData<T>,
    as_null: bool,
}

impl<T: Default> MaybeNull<T> {
    pub fn expr() -> Self {
        Self {
            expr: PhantomData,
            as_null: false,
        }
    }

    pub fn as_null() -> Self {
        Self {
            expr: PhantomData,
            as_null: true,
        }
    }
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
    T: QueryFragment<DB> + Default + Column,
{
    fn walk_ast(&self, mut pass: AstPass<DB>) -> QueryResult<()> {
        if self.as_null {
            pass.push_sql("NULL");
        } else {
            T::default().walk_ast(pass)?;
        }
        Ok(())
    }
}

impl<A, B, DB> QueryFragment<DB> for MaybeNull<MultipleColumnHelper<(A, B)>>
where
    DB: Backend,
    (A, B): QueryFragment<DB> + Default,
{
    fn walk_ast(&self, mut pass: AstPass<DB>) -> QueryResult<()> {
        if self.as_null {
            pass.push_sql("NULL, NULL");
        } else {
            <(A, B) as Default>::default().walk_ast(pass)?;
        }
        Ok(())
    }
}

#[derive(Default, Debug, Clone, Copy)]
pub struct MultipleColumnHelper<T>(T);

impl<T> Expression for MultipleColumnHelper<T>
where
    T: Expression,
{
    type SqlType = T::SqlType;
}
impl<T> NonAggregate for MultipleColumnHelper<T> where T: NonAggregate {}
impl<QS, T> AppearsOnTable<QS> for MultipleColumnHelper<T> where T: AppearsOnTable<QS> {}
impl<QS, T> SelectableExpression<QS> for MultipleColumnHelper<T> where T: SelectableExpression<QS> {}
impl<T, DB> QueryFragment<DB> for MultipleColumnHelper<T>
where
    DB: Backend,
    T: QueryFragment<DB>,
{
    fn walk_ast(&self, pass: AstPass<DB>) -> QueryResult<()> {
        self.0.walk_ast(pass)
    }
}


impl<T> NonAggregate for MaybeNull<T> {}

impl<T, QS> AppearsOnTable<QS> for MaybeNull<T> where Self: Expression {}

impl<T, ST> SelectableExpression<T> for MaybeNull<ST> where Self: Expression {}

