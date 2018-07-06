use diesel::backend::Backend;
use diesel::expression::NonAggregate;
use diesel::query_builder::QueryFragment;
use diesel::{AppearsOnTable, Expression};

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
{}
