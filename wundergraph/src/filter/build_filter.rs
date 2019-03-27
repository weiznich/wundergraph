use crate::diesel_ext::BoxableFilter;
use diesel::backend::Backend;
use diesel::expression::{Expression, NonAggregate, SqlLiteral};
use diesel::query_builder::QueryFragment;
use diesel::sql_types::Bool;

/// A trait that indicates that some type could be converted into a sql filter
/// operation.
pub trait BuildFilter<DB>
where
    DB: Backend,
{
    /// The return type of the constructed filter
    type Ret: Expression<SqlType = ::diesel::sql_types::Bool> + NonAggregate + QueryFragment<DB>;

    /// A function that convertes a given type into a diesel filter expression
    fn into_filter(self) -> Option<Self::Ret>;
}

impl<'a, T, DB> BuildFilter<DB>
    for Box<dyn BoxableFilter<T, DB, SqlType = ::diesel::sql_types::Bool> + 'a>
where
    DB: Backend,
{
    type Ret = Self;
    fn into_filter(self) -> Option<Self::Ret> {
        Some(self)
    }
}

#[cfg_attr(feature = "cargo-clippy", allow(use_self))]
impl<T, DB> BuildFilter<DB> for Option<T>
where
    T: BuildFilter<DB>,
    DB: Backend,
{
    type Ret = T::Ret;

    fn into_filter(self) -> Option<Self::Ret> {
        self.and_then(BuildFilter::into_filter)
    }
}

impl<DB> BuildFilter<DB> for ()
where
    DB: Backend,
{
    type Ret = SqlLiteral<Bool>;

    fn into_filter(self) -> Option<Self::Ret> {
        None
    }
}

// impl<DB, T> BuildFilter<DB> for Box<T>
// where
//     T: BuildFilter<DB> + Sized,
//     DB: Backend,
// {
//     type Ret = T::Ret;

//     fn into_filter(self) -> Option<Self::Ret> {
//         T::into_filter(*self)
//     }
// }

// impl<DB, T> BuildFilter<DB> for T where DB: Backend, T: Expression<SqlType = ::diesel::sql_types::Bool> + NonAggregate + QueryFragment<DB> {

//     type Ret = T;

//     fn into_filter(self) -> Option<Self::Ret> {
//         Some(self)
//     }
// }
