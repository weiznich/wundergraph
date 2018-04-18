use diesel::backend::Backend;
use diesel::expression::{BoxableExpression, Expression, NonAggregate, SqlLiteral};
use diesel::query_builder::QueryFragment;
use diesel::sql_types::Bool;
use filter::transformator::Transformator;

pub trait BuildFilter<DB>
where
    DB: Backend,
{
    type Ret: Expression<SqlType = ::diesel::sql_types::Bool> + NonAggregate + QueryFragment<DB>;

    fn into_filter<T>(self, t: T) -> Option<Self::Ret>
    where
        T: Transformator;
}

impl<'a, T, DB> BuildFilter<DB>
    for Box<BoxableExpression<T, DB, SqlType = ::diesel::sql_types::Bool> + 'a>
where
    DB: Backend,
{
    type Ret = Self;
    fn into_filter<C>(self, _t: C) -> Option<Self::Ret>
    where
        C: Transformator,
    {
        Some(self)
    }
}

impl<T, DB> BuildFilter<DB> for Option<T>
where
    T: BuildFilter<DB>,
    DB: Backend,
{
    type Ret = T::Ret;

    fn into_filter<C>(self, t: C) -> Option<Self::Ret>
    where
        C: Transformator,
    {
        self.and_then(|i| i.into_filter(t))
    }
}

impl<DB> BuildFilter<DB> for ()
where
    DB: Backend,
{
    type Ret = SqlLiteral<Bool>;

    fn into_filter<C>(self, _t: C) -> Option<Self::Ret> {
        None
    }
}
