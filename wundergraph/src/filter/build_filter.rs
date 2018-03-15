use diesel::expression::{BoxableExpression, Expression, NonAggregate, SqlLiteral};
use filter::transformator::Transformator;
use diesel::sql_types::Bool;

pub trait BuildFilter {
    type Ret: Expression<SqlType = ::diesel::sql_types::Bool> + NonAggregate;

    fn into_filter<T>(self, t: T) -> Option<Self::Ret>
    where
        T: Transformator;
}

impl<'a, T, DB> BuildFilter
    for Box<BoxableExpression<T, DB, SqlType = ::diesel::sql_types::Bool> + 'a>
{
    type Ret = Self;
    fn into_filter<C>(self, _t: C) -> Option<Self::Ret>
    where
        C: Transformator,
    {
        Some(self)
    }
}

impl<T> BuildFilter for Option<T>
where
    T: BuildFilter,
{
    type Ret = T::Ret;

    fn into_filter<C>(self, t: C) -> Option<Self::Ret>
    where
        C: Transformator,
    {
        self.and_then(|i| i.into_filter(t))
    }
}

// impl<T> BuildFilter for Box<T>
// where
//     T: BuildFilter,
// {
//     type Ret = T::Ret;

//     fn into_filter<C>(self, t: C) -> Option<Self::Ret>
//     where
//         C: Transformator,
//     {
//         T::into_filter(*self, t)
//     }
// }

impl BuildFilter for () {
    type Ret = SqlLiteral<Bool>;

    fn into_filter<C>(self, _t: C) -> Option<Self::Ret> {
        None
    }
}
